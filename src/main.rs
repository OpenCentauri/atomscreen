// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{cmp::Ordering, error::Error, fs, path::PathBuf, process::exit, rc::Rc, sync::Arc};

use clap::Parser;
use moonraker_rs::{
    connector::websocket_read::{MoonrakerEvent, PrinterEvent},
    moonraker_connection::WebsocketEvent,
    requests::FileManagementRequestHandler,
};
use slint::{Image, Model, ModelRc, Rgba8Pixel, SharedPixelBuffer, SharedString, VecModel};
use tokio::sync::Mutex;

use crate::{config::MoonrakerConfig, event_loop::EventLoop, hardware::init_display};

mod application_error;
mod config;
mod hardware;
mod event_loop;

slint::include_modules!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = config::Args::parse();

    let config_path = PathBuf::from(&args.config);
    if !config_path.exists() {
        eprintln!("Config file does not exist: {}", config_path.display());
        exit(2);
    }

    let config_str = fs::read_to_string(&config_path).unwrap();
    let config = toml::from_str::<config::Config>(&config_str).unwrap();
    let moonraker_config = config.moonraker.unwrap_or(MoonrakerConfig::default());

    let moonraker_connection = Arc::new(
        moonraker_rs::moonraker_connection::MoonrakerConnection::new(
            &moonraker_config.host,
            moonraker_config.port,
        ),
    );
    let ui = init_display(&config.display)?;
    ui.global::<AppState>().set_moonraker_connected(false);
    let ui_weak = ui.as_weak();
    let mut event_loop = EventLoop::new(ui_weak.clone(), moonraker_connection.clone());


    let moonraker_connection_clone = moonraker_connection.clone();
    let moonraker_connection_clone_2 = moonraker_connection.clone();

    let mut receiver = moonraker_connection.get_listener();

    
    let ui_weak_2 = ui.as_weak();
    let ui_weak_3 = ui.as_weak();

    tokio::spawn(async move {
        moonraker_connection.connection_loop().await;
    });

    tokio::spawn(async move {
        event_loop.event_loop().await;
    });

    ui.global::<Filesystem>().on_list_files(move || {
        println!("List files clicked");
        let moonraker_connection = moonraker_connection_clone.clone();
        let ui_weak = ui_weak_2.clone();
        ui_weak_2
            .upgrade()
            .unwrap()
            .global::<Filesystem>()
            .set_loading(true);
        slint::spawn_local(async move {
            if let Ok(mut files) = moonraker_connection.list_gcode_files().await {
                files.sort_by(|a, b| {
                    if a.modified > b.modified {
                        Ordering::Less
                    } else if a.modified < b.modified {
                        Ordering::Greater
                    } else {
                        Ordering::Equal
                    }
                });

                println!("Files: {:?}", files);

                let ui = ui_weak.upgrade().unwrap();

                let converted_files: Vec<MoonrakerFileTest> = files
                    .iter()
                    .map(|f| MoonrakerFileTest {
                        path: SharedString::from(&f.path),
                        modified: f.modified,
                        size: f.size,
                        permissions: SharedString::from(&f.permissions),
                        thumbnail: Image::default(),
                    })
                    .collect();

                ui.global::<Filesystem>()
                    .set_files(ModelRc::new(Rc::new(VecModel::from(converted_files))));
                ui.global::<Filesystem>().set_loading(false);
                ui.global::<Filesystem>().invoke_download_thumbnail(0);
                ui.global::<Filesystem>().invoke_download_thumbnail(1);
                ui.global::<Filesystem>().invoke_download_thumbnail(2);
                ui.global::<Filesystem>().invoke_download_thumbnail(3);
                ui.global::<Filesystem>().invoke_download_thumbnail(4);
            }
        })
        .unwrap();
    });

    ui.global::<Utils>().on_image_exists(|f| {
        return f.size().area() > 0;
    });

    let mutex = Arc::new(Mutex::new(()));

    ui.global::<Filesystem>()
        .on_download_thumbnail(move |global_index| {
            let mutex2 = mutex.clone();
            let ui_weak = ui_weak_3.clone();
            let moonraker_connection = moonraker_connection_clone_2.clone();
            slint::spawn_local(async move {
                let global_index = global_index as usize;
                let mutex = mutex2.lock().await;
                let ui = ui_weak.upgrade().unwrap();

                let f = ui.global::<Filesystem>().get_files();
                let files = f.as_any().downcast_ref::<VecModel<MoonrakerFileTest>>();

                let non_optional_files = match files {
                    Some(files) => files,
                    None => return,
                };

                let mut unwrapped_files: Vec<MoonrakerFileTest> =
                    non_optional_files.iter().collect();

                if global_index < unwrapped_files.len() {
                    let file = &mut unwrapped_files[global_index];

                    if file.thumbnail.size().area() > 0 {
                        // Already have a thumbnail
                        return;
                    }

                    // TODO: Error handling
                    let files = moonraker_connection
                        .get_thumbnails_for_file(&file.path)
                        .await
                        .unwrap();
                    println!("Thumbnails for {}: {:?}", file.path, files);
                    let possible_target =
                        files.into_iter().find(|f| f.width == 32 && f.height == 32);

                    if let Some(target) = possible_target {
                        let data = moonraker_connection
                            .download_thumbnail(&target.thumbnail_path)
                            .await
                            .unwrap();

                        // TODO: Error handling
                        let image = image::load_from_memory(&data).unwrap().into_rgba8();
                        let shared_buf: SharedPixelBuffer<Rgba8Pixel> =
                            SharedPixelBuffer::clone_from_slice(
                                image.as_raw(),
                                image.width(),
                                image.height(),
                            );

                        file.thumbnail = Image::from_rgba8(shared_buf);
                        println!("Set thumbnail for file {}", file.path);
                        non_optional_files.set_vec(unwrapped_files);
                    }
                }
            })
            .unwrap();
        });

    ui.global::<Utils>().on_format_bytes(|bytes| {
        if bytes < 1024 {
            return SharedString::from(format!("{} B", bytes));
        }

        let units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
        let idx = (bytes as f64).log(1024.0).floor() as usize;
        let value = bytes as f64 / 1024_f64.powi(idx as i32);

        SharedString::from(format!("{:.2} {}", value, units[idx]))
    });

    tokio::task::block_in_place(|| {
        ui.run().unwrap();
    });

    Ok(())
}
