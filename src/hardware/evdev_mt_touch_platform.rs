use std::{cell::RefCell, sync::Mutex};

use evdev::{AbsoluteAxisCode, EventSummary, KeyCode};
use slint::{
    platform::{PointerEventButton, WindowEvent},
    LogicalPosition,
};

use crate::hardware::TouchPlatform;

pub struct EvdevMtTouchPlatform {
    touch_device: RefCell<evdev::Device>,
    last_touch: RefCell<LogicalPosition>,
}

impl EvdevMtTouchPlatform {
    pub fn new(touch_device: evdev::Device) -> Self {
        Self {
            touch_device: RefCell::new(touch_device),
            last_touch: RefCell::new(LogicalPosition::new(0.0, 0.0)),
        }
    }
}

impl TouchPlatform for EvdevMtTouchPlatform {
    fn process_touch_events(&self) -> Vec<WindowEvent> {
        let mut touch_device = self.touch_device.borrow_mut();
        let events = touch_device.fetch_events();
        let mut result = vec![];
        let mut last_touch = self.last_touch.borrow_mut();
        let mut pos_updated = false;
        let mut touch_started = false;
        let mut touch_ended = false;

        if let Ok(unwrapped_events) = events {
            for event in unwrapped_events {
                match event.destructure() {
                    EventSummary::Key(ev, KeyCode::BTN_TOUCH, 1) => {
                        touch_started = true;
                    }
                    EventSummary::Key(ev, KeyCode::BTN_TOUCH, 0) => {
                        touch_ended = true;
                    }
                    EventSummary::AbsoluteAxis(ev, AbsoluteAxisCode::ABS_MT_POSITION_X, value) => {
                        pos_updated = true;
                        last_touch.x = value as f32;
                    }
                    EventSummary::AbsoluteAxis(ev, AbsoluteAxisCode::ABS_MT_POSITION_Y, value) => {
                        pos_updated = true;
                        last_touch.y = value as f32;
                    }
                    _ => {}
                };
            }

            // TODO: Evdev is a stream of multiple events, we're swallowing events here.
            if touch_ended {
                if !pos_updated {
                    eprintln!("Warn on evdev_mt_touch_platform: Touch ended without receiving coordinates");
                }

                result.push(WindowEvent::PointerReleased {
                    position: last_touch.clone(),
                    button: PointerEventButton::Left,
                });
            } else if touch_started {
                if !pos_updated {
                    eprintln!("Warn on evdev_mt_touch_platform: Touch started without receiving coordinates");
                }

                result.push(WindowEvent::PointerPressed {
                    position: last_touch.clone(),
                    button: PointerEventButton::Left,
                });
            } else if pos_updated {
                result.push(WindowEvent::PointerMoved {
                    position: last_touch.clone(),
                })
            }
        }

        result
    }
}
