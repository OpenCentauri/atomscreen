use std::{collections::HashMap, sync::Arc};

use moonraker_rs::{moonraker_connection::MoonrakerConnection, requests::PrinterAdministrationRequestHandler};
use slint::{ComponentHandle, ModelRc, SharedString, SharedVector, VecModel};

use crate::{AppWindow, QuickActions};

pub fn register_execute_quick_action(ui: &AppWindow, quick_actions: &Option<HashMap<String, Vec<String>>>, moonraker_connection: &Arc<MoonrakerConnection>)
{
    let mut quick_actions_map: HashMap<String, String> = HashMap::new();
    let moonraker_connection = moonraker_connection.clone();

    if let Some(quick_actions) = quick_actions 
    {
        quick_actions.iter()
        .for_each(|f| {
            quick_actions_map.insert(
                f.0.clone(), 
                f.1.join("\n"),
            );
        });
    }

    let keys : Vec<SharedString> = quick_actions_map.keys().map(|f| SharedString::from(f)).collect();
    ui.global::<QuickActions>().set_quick_actions(ModelRc::new(VecModel::from(keys)));
    ui.global::<QuickActions>().on_execute_quick_action(move |f| {
        let moonraker_connection = moonraker_connection.clone();
        let gcode = match quick_actions_map.get(f.as_str())
        {
            Some(s) => s.clone(),
            None => return
        };

        tokio::spawn(async move {
            if let Err(e) = moonraker_connection.run_gcode_script(&gcode).await
            {
                moonraker_connection.send_request_error(format!("Failed to send quick action '{}': {}", f.as_str(), e));
            }
        });
    });
}