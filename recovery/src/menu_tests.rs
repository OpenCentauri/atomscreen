use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::mpsc::{self, Sender},
};

use slint::{
    ComponentHandle, LogicalPosition, Model, PhysicalSize, Rgb8Pixel,
    platform::{
        EventLoopProxy, Platform, PointerEventButton, WindowAdapter, WindowEvent,
        software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
    },
};

use crate::{AppWindow, State, config, task_exec, to_action_model, to_list};

type Event = Box<dyn FnOnce() + Send>;

struct Offscreen {
    window: Rc<MinimalSoftwareWindow>,
    events: Sender<Event>,
}

struct Proxy(Sender<Event>);

impl Platform for Offscreen {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }

    fn new_event_loop_proxy(&self) -> Option<Box<dyn EventLoopProxy>> {
        Some(Box::new(Proxy(self.events.clone())))
    }
}

impl EventLoopProxy for Proxy {
    fn quit_event_loop(&self) -> Result<(), slint::EventLoopError> {
        Ok(())
    }

    fn invoke_from_event_loop(&self, event: Event) -> Result<(), slint::EventLoopError> {
        self.0.send(event).unwrap();
        Ok(())
    }
}

fn render(window: &MinimalSoftwareWindow, name: &str) {
    let mut pixels = vec![Rgb8Pixel::default(); 480 * 272];
    window.window().request_redraw();
    assert!(window.draw_if_needed(|renderer| { renderer.render(&mut pixels, 480); }));
    assert!(pixels.iter().any(|pixel| *pixel != Rgb8Pixel::default()));
    if let Ok(dir) = std::env::var("ATOMSCREEN_MENU_SCREENSHOT_DIR") {
        let mut ppm = b"P6\n480 272\n255\n".to_vec();
        for pixel in pixels {
            ppm.extend_from_slice(&[pixel.r, pixel.g, pixel.b]);
        }
        std::fs::write(std::path::Path::new(&dir).join(format!("{name}.ppm")), ppm).unwrap();
    }
}

// Sweep only the Run column: an enabled control must fire, disabled entries must not.
fn click_run_column(ui: &AppWindow) {
    for y in (24..248).step_by(8) {
        let position = LogicalPosition::new(420.0, y as f32);
        ui.window().dispatch_event(WindowEvent::PointerPressed {
            position,
            button: PointerEventButton::Left,
        });
        ui.window().dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
    }
}

#[test]
fn root_and_submenu_keep_disabled_notes_and_block_clicks_and_execution() {
    let config: config::Config = toml::from_str(r#"
        [display.default]
        width = 480
        height = 272

        [scripts."Install Root Shell"]
        index = 0
        disabled = true
        disabled_reason = "Running OC-patched, root shell already present"
        tasks = [{ name = "Must not run", command = [] }]

        [scripts."Root enabled control"]
        index = 1
        tasks = []

        [submenus.Utilities."Uninstall Root Shell"]
        index = 0
        disabled = true
        disabled_reason = "Running OC-patched, root shell already present"
        tasks = [{ name = "Must not run", command = [] }]

        [submenus.Utilities."Submenu enabled control"]
        index = 1
        tasks = []
    "#).unwrap();
    let root = to_list(config.scripts);
    let submenus: HashMap<_, _> = config.submenus.unwrap().into_iter()
        .map(|(name, scripts)| (name, to_list(scripts))).collect();
    let reason = "Running OC-patched, root shell already present";
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    let (sender, events) = mpsc::channel();
    slint::platform::set_platform(Box::new(Offscreen {
        window: window.clone(), events: sender,
    })).unwrap();
    let ui = AppWindow::new().unwrap();
    window.set_size(PhysicalSize::new(480, 272));
    let state = ui.global::<State>();
    state.set_actions(to_action_model(&root));
    state.set_window_title("CC1 utilities".into());
    // The group menu uses the same ActionEntry but is not itself a runnable script.
    state.set_submenus(crate::to_shared_string_model(["Utilities".into()]));
    crate::register_action_submenu_enter(&ui, &submenus);
    crate::register_action_exit_submenu(&ui, &root, &["Utilities".into()], "CC1 utilities");
    ui.show().unwrap();

    let clicks = Rc::new(RefCell::new(Vec::new()));
    let clicked = clicks.clone();
    state.on_action_start(move |name| clicked.borrow_mut().push(name.to_string()));
    for (name, control, expected_back) in [
        ("root-disabled", "Root enabled control", false),
        ("submenu-disabled", "Submenu enabled control", true),
        ("root-restored", "Root enabled control", false),
    ] {
        let model = state.get_actions();
        let disabled = model.row_data(0).unwrap();
        assert!(disabled.disabled);
        assert_eq!(disabled.disabled_reason.as_str(), reason);
        assert!(!model.row_data(1).unwrap().disabled);
        assert!(model.row_data(1).unwrap().disabled_reason.is_empty());
        assert_eq!(state.get_show_back(), expected_back);
        render(&window, name);
        // Hide group buttons while sweeping so navigation cannot change the view.
        let groups = state.get_submenus();
        state.set_submenus(crate::to_shared_string_model(Vec::new()));
        render(&window, "click-targets");
        clicks.borrow_mut().clear();
        click_run_column(&ui);
        assert!(!clicks.borrow().is_empty(), "enabled control was not clickable");
        assert!(clicks.borrow().iter().all(|clicked| clicked == control));
        state.set_submenus(groups);
        if expected_back {
            state.invoke_action_exit_submenu();
        } else if name == "root-disabled" {
            state.invoke_action_submenu_enter("Utilities".into());
        }
        for event in events.try_iter() { event(); }
    }
    assert_eq!(state.get_window_title().as_str(), "CC1 utilities");
    assert_eq!(state.get_submenus().row_data(0).unwrap().as_str(), "Utilities");

    let marker = std::env::temp_dir().join(format!("atomscreen-disabled-command-{}", std::process::id()));
    assert!(!marker.exists());
    for mut script in [root[0].clone(), submenus["Utilities"][0].clone()] {
        script.tasks[0].command = vec!["/usr/bin/touch".into(), marker.to_str().unwrap().into()];
        task_exec::process_script_tasks(ui.as_weak(), script);
        assert!(!marker.exists(), "disabled script ran its command");
        assert_eq!(events.try_iter().count(), 0, "disabled script published UI state");
        assert!(state.get_active_action().is_empty());
        assert_eq!(state.get_active_action_state(), 0);
        assert_eq!(state.get_active_action_tasks().row_count(), 0);
        assert_eq!(state.get_active_action_buttons().row_count(), 0);
    }
}
