use std::{cmp::Ordering, rc::Rc, sync::Arc};

use moonraker_rs::{moonraker_connection::MoonrakerConnection, requests::FileManagementRequestHandler};
use slint::{ComponentHandle, Image, ModelRc, SharedString, VecModel};

use crate::{AppWindow, Filesystem, MoonrakerFile};

pub fn register_filesystem_list_files(ui : &AppWindow, moonraker_connection : &Arc<MoonrakerConnection>)
{
    let ui_weak = ui.as_weak();
    let moonraker_connection = moonraker_connection.clone();
    ui.global::<Filesystem>().on_list_files(move || {
        let moonraker_connection = moonraker_connection.clone();
        let ui_weak = ui_weak.clone();
        ui_weak
            .upgrade()
            .unwrap()
            .global::<Filesystem>()
            .set_loading(true);
        slint::spawn_local(async move {
            let mut files = match moonraker_connection.list_gcode_files().await
            {
                Ok(f) => f,
                Err(e) => {
                    moonraker_connection.send_request_error(format!("Failed to list files: {}", e));
                    return;
                }
            };

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

            let converted_files: Vec<MoonrakerFile> = files
                .iter()
                .map(|f| MoonrakerFile {
                    path: SharedString::from(&f.path),
                    modified: f.modified,
                    size: f.size,
                    permissions: SharedString::from(&f.permissions),
                    ..MoonrakerFile::default()
                })
                .collect();

            ui.global::<Filesystem>()
                .set_files(ModelRc::new(Rc::new(VecModel::from(converted_files))));
            ui.global::<Filesystem>().set_loading(false);
            for i in 0..5 {
                ui.global::<Filesystem>().invoke_fetch_metadata(i);
            }
        })
        .unwrap();
    });
}