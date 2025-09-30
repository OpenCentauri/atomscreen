use std::{cmp::Ordering, rc::Rc, sync::Arc};

use moonraker_rs::{moonraker_connection::MoonrakerConnection, requests::FileManagementRequestHandler};
use slint::{ComponentHandle, Image, ModelRc, SharedString, VecModel};

use crate::{AppWindow, Filesystem, MoonrakerFileTest};

pub fn register_filesystem_list_files(ui : &AppWindow, moonraker_connection : &Arc<MoonrakerConnection>)
{
    let ui_weak = ui.as_weak();
    let moonraker_connection = moonraker_connection.clone();
    ui.global::<Filesystem>().on_list_files(move || {
        println!("List files clicked");
        let moonraker_connection = moonraker_connection.clone();
        let ui_weak = ui_weak.clone();
        ui_weak
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
}