use std::sync::Arc;

use moonraker_rs::{moonraker_connection::{MoonrakerConnection}, requests::FileManagementRequestHandler};
use slint::{ComponentHandle, Image, Model, Rgba8Pixel, SharedPixelBuffer, SharedString, VecModel};
use tokio::sync::Mutex;

use crate::{AppWindow, Filesystem, MoonrakerFile};

pub fn register_filesystem_fetch_metadata(ui: &AppWindow, moonraker_connection : &Arc<MoonrakerConnection>)
{
    let mutex = Arc::new(Mutex::new(()));
    let ui_weak = ui.as_weak();
    let moonraker_connection = moonraker_connection.clone();

    ui.global::<Filesystem>()
        .on_fetch_metadata(move |global_index| {
            let mutex = mutex.clone();
            let ui_weak = ui_weak.clone();
            let moonraker_connection = moonraker_connection.clone();
            slint::spawn_local(async move {
                let global_index = global_index as usize;
                let mutex = mutex.lock().await;
                let ui = ui_weak.upgrade().unwrap();

                let f = ui.global::<Filesystem>().get_files();
                let files = f.as_any().downcast_ref::<VecModel<MoonrakerFile>>();

                let non_optional_files = match files {
                    Some(files) => files,
                    None => return,
                };

                let mut unwrapped_files: Vec<MoonrakerFile> =
                    non_optional_files.iter().collect();

                if global_index < unwrapped_files.len() {
                    let file = &mut unwrapped_files[global_index];

                    if file.thumbnail.size().area() > 0 {
                        // Already have a thumbnail
                        return;
                    }

                    // TODO: Error handling
                    let metadata = moonraker_connection
                        .get_gcode_metadata_for_file(&file.path)
                        .await
                        .unwrap();

                    file.filament_used_gram = metadata.filament_weight_total;
                    file.filament_type = SharedString::from(&metadata.filament_type);
                    file.height_mm = metadata.object_height;
                    file.estimated_time_s = metadata.estimated_time;
                    file.layer_height_mm = metadata.layer_height;
                    file.nozzle_diameter_mm = metadata.nozzle_diameter;

                    let files = metadata.absolute_thumbnails();
                    println!("Thumbnails for {}: {:?}", file.path, files);
                    let possible_target = files.into_iter().find(|f| f.width == 32 && f.height == 32);

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
                    }

                    non_optional_files.set_vec(unwrapped_files);
                }
            })
            .unwrap();
        });
}