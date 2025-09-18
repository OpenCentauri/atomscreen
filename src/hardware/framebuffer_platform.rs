// Based on https://github.com/nilclass/slint-framebuffer-example

use std::{cell::RefCell, rc::Rc, sync::{Arc, Mutex}, time::{Duration, Instant}, collections::VecDeque};
use evdev::Device;
use linuxfb::{double::Buffer, Framebuffer};
use slint::{platform::{software_renderer::{MinimalSoftwareWindow, PremultipliedRgbaColor, RepaintBufferType, Rgb565Pixel, TargetPixel}, Platform, WindowEvent}, PhysicalSize, Rgb8Pixel};

use crate::hardware::EvdevMtTouchPlatform;

pub trait TouchPlatform
{
    fn process_touch_events(&self) -> Vec<WindowEvent>;
}

pub struct FramebufferPlatform {
    window: Rc<MinimalSoftwareWindow>,
    fb: RefCell<Buffer>,
    width: usize,
    height: usize,
    stride: usize,
    bytes_per_pixel: usize,
    touch_device: Option<Box<dyn TouchPlatform>>,
    event_queue: Arc<Mutex<VecDeque<Box<dyn FnOnce() + Send>>>>,
    quit_requested: Arc<Mutex<bool>>,
}

impl FramebufferPlatform {
    pub fn new(fb : Framebuffer, touch_device : Option<Device>) -> Self {
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
        
        let window = MinimalSoftwareWindow::new(RepaintBufferType::SwappedBuffers);
        window.set_size(PhysicalSize::new(size.0, size.1));

        Self {
            window,
            fb: RefCell::new(Buffer::new(fb).unwrap()),
            width: size.0 as usize,
            height: size.1 as usize,
            stride: size.0 as usize,
            bytes_per_pixel: bytes_per_pixel as usize,
            touch_device: mutex_touch_device,
            event_queue: Arc::new(Mutex::new(VecDeque::new())),
            quit_requested: Arc::new(Mutex::new(false)),
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
        Self { red: r, green: g, blue: b, alpha: 255 }
    }

    fn background() -> Self {
        Self { red: 0, green: 0, blue: 0, alpha: 0 }
    }
}

struct FramebufferEventLoopProxy {
    event_queue: Arc<Mutex<VecDeque<Box<dyn FnOnce() + Send>>>>,
    quit_requested: Arc<Mutex<bool>>,
}

impl FramebufferEventLoopProxy {
    fn new(event_queue: Arc<Mutex<VecDeque<Box<dyn FnOnce() + Send>>>>, quit_requested: Arc<Mutex<bool>>) -> Self {
        Self { event_queue, quit_requested }
    }
}

impl slint::platform::EventLoopProxy for FramebufferEventLoopProxy {
    fn quit_event_loop(&self) -> Result<(), slint::EventLoopError> {
        if let Ok(mut quit_flag) = self.quit_requested.lock() {
            *quit_flag = true;
            Ok(())
        } else {
            Err(slint::EventLoopError::EventLoopTerminated)
        }
    }

    fn invoke_from_event_loop(
        &self,
        event: Box<dyn FnOnce() + Send>,
    ) -> Result<(), slint::EventLoopError> {
        if let Ok(mut queue) = self.event_queue.lock() {
            queue.push_back(event);
            Ok(())
        } else {
            Err(slint::EventLoopError::EventLoopTerminated)
        }
    }
}

impl Platform for FramebufferPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }

    fn new_event_loop_proxy(&self) -> Option<Box<dyn slint::platform::EventLoopProxy>> {
        Some(Box::new(FramebufferEventLoopProxy::new(
            self.event_queue.clone(),
            self.quit_requested.clone()
        )))
    }

    fn run_event_loop(&self) -> Result<(), slint::PlatformError> {
        let mut fb = self.fb.borrow_mut();
        loop {
            // Check for quit request
            if let Ok(quit_flag) = self.quit_requested.lock() {
                if *quit_flag {
                    break;
                }
            }

            // Process queued events from other threads
            if let Ok(mut queue) = self.event_queue.lock() {
                while let Some(event) = queue.pop_front() {
                    event();
                }
            }

            slint::platform::update_timers_and_animations();

            if let Some(touch_device) = &self.touch_device
            {
                let now = Instant::now();
                let events = touch_device.process_touch_events();
                let elapsed = now.elapsed();

                for event in events
                {
                    println!("Got event {:?}", event);
                    println!("Elapsed: {:.2?}", elapsed);
                    self.window.try_dispatch_event(event).unwrap();
                }
            }

            self.window.draw_if_needed(|renderer| {
                let frame: &mut[u8] = fb.as_mut_slice();
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

            if !self.window.has_active_animations() {
                std::thread::sleep(slint::platform::duration_until_next_timer_update().unwrap_or(Duration::from_millis(100)));
            }
        }
        Ok(())
    }
}