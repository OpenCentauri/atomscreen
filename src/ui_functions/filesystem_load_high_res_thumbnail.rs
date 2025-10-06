use std::sync::Arc;

use moonraker_rs::{moonraker_connection::MoonrakerConnection, requests::{FileManagementRequestHandler, MoonrakerFileThumbnail}};
use slint::{ComponentHandle, Image, Model, Rgba8Pixel, SharedPixelBuffer, SharedString, VecModel};
use tokio::sync::Mutex;

use crate::{AppWindow, Filesystem, MoonrakerFile};

pub fn register_filesystem_load_high_res_thumbnail(ui: &AppWindow, moonraker_connection : &Arc<MoonrakerConnection>)
{
    let ui_weak = ui.as_weak();
    let moonraker_connection = moonraker_connection.clone();
 
    ui.global::<Filesystem>()
        .on_load_high_res_thumbnail(move |file_path| {
            let ui_weak = ui_weak.clone();
            let moonraker_connection = moonraker_connection.clone();

            slint::spawn_local(async move {
                let thumbnails = match moonraker_connection
                    .get_thumbnails_for_file(&file_path)
                    .await
                {
                    Ok(t) => t,
                    Err(e) => {
                        moonraker_connection.send_request_error(format!("Failed to get thumbnails for file {}: {}", file_path, e));
                        return;
                    }
                };

                let mut selected_thumbnail : MoonrakerFileThumbnail = MoonrakerFileThumbnail {
                    width: 0,
                    height: 0,
                    size: 0,
                    thumbnail_path: String::from(""),
                };

                for thumbnail in thumbnails {
                    // No non-square thumbnails
                    if thumbnail.height != thumbnail.width
                    {
                        continue;
                    }

                    if thumbnail.height > 256
                    {
                        continue;
                    }

                    if selected_thumbnail.width < thumbnail.width
                    {
                        selected_thumbnail = thumbnail;
                    }
                }

                if selected_thumbnail.width <= 0
                {
                    return;
                }

                let data = moonraker_connection
                    .download_thumbnail(&selected_thumbnail.thumbnail_path)
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

                let ui = ui_weak.upgrade().unwrap();
                ui.global::<Filesystem>().set_high_res_thumbnail(Image::from_rgba8(shared_buf));                
            })
            .unwrap();
        });
}