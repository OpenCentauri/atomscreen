use evdev::Device;
use linuxfb::Framebuffer;
use slint::platform::Platform;

use crate::framebuffer::framebuffer_platform::FramebufferPlatform;

pub fn init_framebuffer(fb_path: String, event_path: Option<String>, double_buffering: bool) -> Box<dyn Platform + 'static> {
    let fb = Framebuffer::new(fb_path).expect("Failed to initialise framebuffer");
    
    let touch_device = match event_path {
        Some(path) => Some(Device::open(&path).expect("Failed to open touch event device")),
        None => None,
    };

    Box::new(FramebufferPlatform::new(fb, touch_device, double_buffering))
}