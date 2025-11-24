// Based on https://github.com/nilclass/slint-framebuffer-example

use evdev::Device;
use linuxfb::{double::Buffer, Framebuffer};
use memmap::MmapMut;
use slint::{
    platform::{
        software_renderer::{
            MinimalSoftwareWindow, PremultipliedRgbaColor, RepaintBufferType, Rgb565Pixel,
            TargetPixel,
        },
        EventLoopProxy, Platform, WindowEvent,
    },
    EventLoopError, PhysicalSize, PlatformError, Rgb8Pixel,
};
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

use crate::framebuffer::evdev_mt_touch_platform::EvdevMtTouchPlatform;

trait FramebufferHandler
{
    fn as_mut_slice(&mut self) -> &mut [u8];
    fn flip(&mut self) -> Result<(), linuxfb::Error>;
}

struct SingleBufferFramebuffer
{
    fb: Framebuffer,
    map: MmapMut,
}

impl SingleBufferFramebuffer
{
    fn new(fb : Framebuffer) -> Result<SingleBufferFramebuffer, linuxfb::Error>
    {
        let map = fb.map()?;
        Ok(SingleBufferFramebuffer { fb: fb, map: map })
    }
}

impl FramebufferHandler for SingleBufferFramebuffer
{
    fn as_mut_slice(&mut self) -> &mut [u8]
    {
        &mut self.map[..]
    }

    fn flip(&mut self) -> Result<(), linuxfb::Error> {
        // Do nothing
        Ok(())
    }
}

struct DoubleBufferFramebuffer
{
    buffer: Buffer,
}

impl DoubleBufferFramebuffer
{
    fn new(fb : Framebuffer) -> Result<DoubleBufferFramebuffer, linuxfb::Error>
    {
        let buffer = Buffer::new(fb)?;
        Ok(DoubleBufferFramebuffer { buffer: buffer })
    }
}

impl FramebufferHandler for DoubleBufferFramebuffer
{
    fn as_mut_slice(&mut self) -> &mut [u8] {
        self.buffer.as_mut_slice()
    }

    fn flip(&mut self) -> Result<(), linuxfb::Error> {
        self.buffer.flip()
    }
}

pub trait TouchPlatform {
    fn process_touch_events(&self) -> Vec<WindowEvent>;
}

pub struct FramebufferPlatform {
    window: Rc<MinimalSoftwareWindow>,
    fb: RefCell<Box<dyn FramebufferHandler>>,
    width: usize,
    height: usize,
    stride: usize,
    bytes_per_pixel: usize,
    touch_device: Option<Box<dyn TouchPlatform>>,
    queue: Option<Queue>,
}

impl FramebufferPlatform {
    pub fn new(fb: Framebuffer, touch_device: Option<Device>, double_buffering: bool) -> Self {
        let size = fb.get_size();
        let bytes_per_pixel = fb.get_bytes_per_pixel();
        let physical_size = fb.get_physical_size();

        println!("Framebuffer id: {:?}", fb.get_id());
        println!("Size in pixels: {:?}", size);
        println!("Bytes per pixel: {:?}", bytes_per_pixel);
        println!("Physical size in mm: {:?}", physical_size);

        let mut mutex_touch_device: Option<Box<dyn TouchPlatform>> = None;

        if let Some(touch_device) = touch_device {
            println!("Input device name: {:?}", touch_device.name());
            touch_device.set_nonblocking(true).unwrap();

            // TODO: Allow this to be configured
            mutex_touch_device = Some(Box::new(EvdevMtTouchPlatform::new(touch_device)));
        } else {
            println!("No input device configured");
        }

        let window = MinimalSoftwareWindow::new(if double_buffering { RepaintBufferType::SwappedBuffers } else { RepaintBufferType::ReusedBuffer });
        window.set_size(PhysicalSize::new(size.0, size.1));

        let framebuffer_handler: Box<dyn FramebufferHandler> = match double_buffering
        {
            true => Box::new(DoubleBufferFramebuffer::new(fb).expect("Failed to initialise double buffer")),
            false => Box::new(SingleBufferFramebuffer::new(fb).expect("Failed to initialise single buffer"))
        };


        Self {
            window,
            fb: RefCell::new(framebuffer_handler),
            width: size.0 as usize,
            height: size.1 as usize,
            stride: size.0 as usize,
            bytes_per_pixel: bytes_per_pixel as usize,
            touch_device: mutex_touch_device,
            queue: Some(Queue(Default::default(), std::thread::current())),
        }
    }
}

#[derive(Clone, Copy)]
pub struct PremultipliedAbgrColor {
    pub blue: u8,
    pub green: u8,
    pub red: u8,
    pub alpha: u8,
}

impl TargetPixel for PremultipliedAbgrColor {
    fn blend(&mut self, color: PremultipliedRgbaColor) {
        let a = (u8::MAX - color.alpha) as u16;
        self.red = (self.red as u16 * a / 255) as u8 + color.red;
        self.green = (self.green as u16 * a / 255) as u8 + color.green;
        self.blue = (self.blue as u16 * a / 255) as u8 + color.blue;
        self.alpha = (self.alpha as u16 + color.alpha as u16
            - (self.alpha as u16 * color.alpha as u16) / 255) as u8;
    }

    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            red: r,
            green: g,
            blue: b,
            alpha: 255,
        }
    }

    fn background() -> Self {
        Self {
            red: 0,
            green: 0,
            blue: 0,
            alpha: 0,
        }
    }
}

impl Platform for FramebufferPlatform {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }

    fn new_event_loop_proxy(&self) -> Option<Box<dyn slint::platform::EventLoopProxy>> {
        self.queue
            .as_ref()
            .map(|q| Box::new(q.clone()) as Box<dyn EventLoopProxy>)
    }

    fn run_event_loop(&self) -> Result<(), slint::PlatformError> {
        let mut fb = self.fb.borrow_mut();

        let queue = match self.queue.as_ref() {
            Some(queue) => queue.clone(),
            None => return Err(PlatformError::NoEventLoopProvider),
        };

        loop {
            slint::platform::update_timers_and_animations();

            let e = queue.0.lock().unwrap().pop_front();

            match e {
                Some(Event::Quit) => break,
                Some(Event::Event(event)) => event(),
                None => {}
            }

            if let Some(touch_device) = &self.touch_device {
                //let now = Instant::now();
                let events = touch_device.process_touch_events();
                //let has_event = !events.is_empty();

                for event in events {
                    //println!("Got event {:?}", event);
                    self.window.try_dispatch_event(event).unwrap();
                }

                //if has_event {
                //    println!("Elapsed: {:.2?}", now.elapsed());
                //}
            }

            self.window.draw_if_needed(|renderer| {
                let frame = fb.as_mut_slice();
                if self.bytes_per_pixel == 2 {
                    let (_, pixels, _) = unsafe { frame.align_to_mut::<Rgb565Pixel>() };
                    renderer.render(pixels, self.stride);
                } else if self.bytes_per_pixel == 4 {
                    // TODO: This may be different on other machines. Should be configurable!
                    let (_, pixels, _) = unsafe { frame.align_to_mut::<PremultipliedAbgrColor>() };
                    renderer.render(pixels, self.stride);
                } else if self.bytes_per_pixel == 3 {
                    let (_, pixels, _) = unsafe { frame.align_to_mut::<Rgb8Pixel>() };
                    renderer.render(pixels, self.stride);
                } else {
                    panic!("Unsupported framebuffer pixel format");
                }

                fb.flip().unwrap();
            });

            let queue_length = queue.0.lock().unwrap().len();

            if !self.window.has_active_animations() && queue_length <= 0 {
                std::thread::park_timeout(
                    slint::platform::duration_until_next_timer_update()
                        .unwrap_or(Duration::from_millis(20)),
                );
            }
        }
        Ok(())
    }
}

enum Event {
    Quit,
    Event(Box<dyn FnOnce() + Send>),
}

#[derive(Clone)]
struct Queue(
    std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<Event>>>,
    std::thread::Thread,
);

impl EventLoopProxy for Queue {
    fn quit_event_loop(&self) -> Result<(), EventLoopError> {
        self.0.lock().unwrap().push_back(Event::Quit);
        self.1.unpark();
        Ok(())
    }

    fn invoke_from_event_loop(
        &self,
        event: Box<dyn FnOnce() + Send>,
    ) -> Result<(), EventLoopError> {
        self.0.lock().unwrap().push_back(Event::Event(event));
        self.1.unpark();
        Ok(())
    }
}
