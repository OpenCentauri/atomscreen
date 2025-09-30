use std::sync::Arc;

use moonraker_rs::{moonraker_connection::{self, MoonrakerConnection}, requests::FileManagementRequestHandler};
use slint::{ComponentHandle, Image, Model, Rgba8Pixel, SharedPixelBuffer, VecModel};
use tokio::sync::Mutex;

use crate::{AppWindow, Filesystem, MoonrakerFileTest};

pub fn register_filesystem_download_thumbnails(ui: &AppWindow, moonraker_connection : &Arc<MoonrakerConnection>)
{
    let mutex = Arc::new(Mutex::new(()));
    let ui_weak = ui.as_weak();
    let moonraker_connection = moonraker_connection.clone();

    ui.global::<Filesystem>()
        .on_download_thumbnail(move |global_index| {
            let mutex = mutex.clone();
            let ui_weak = ui_weak.clone();
            let moonraker_connection = moonraker_connection.clone();
            slint::spawn_local(async move {
                let global_index = global_index as usize;
                let mutex = mutex.lock().await;
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
}