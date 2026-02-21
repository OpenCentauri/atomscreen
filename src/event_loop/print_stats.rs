use moonraker_rs::{connector::websocket_read::PrinterEvent, printer_objects::PrintState, requests::FileManagementRequestHandler};
use slint::{ComponentHandle, SharedString};

use crate::{PrintStatus, PrintStatusState, application_error::ApplicationError, event_loop::EventLoop};

impl EventLoop {
    pub async fn handle_print_stats_updates(
        &mut self,
        printer_event: &PrinterEvent,
    ) -> Result<(), ApplicationError> {
        if let PrinterEvent::PrintStats(print_stats) = printer_event {
            let total_layers = print_stats.info.total_layer.unwrap_or(0);
            let current_layer = print_stats.info.current_layer.unwrap_or(0);
            let elapsed_time = print_stats.print_duration;
            let filename = SharedString::from(&print_stats.filename);

            if self.last_state == PrintState::Standby && print_stats.state == PrintState::Printing {
                self.slicer_time_estimate = None;
            }

            if self.slicer_time_estimate.is_none() {
                let metadata = self.moonraker_connection.get_gcode_metadata_for_file(&print_stats.filename).await;
                if let Ok(metadata) = metadata {
                    self.slicer_time_estimate = Some(metadata.estimated_time as u64);
                }
                else {
                    self.slicer_time_estimate = Some(0);
                }
            }

            self.last_state = print_stats.state.clone();

            let state = PrintStatusState {
                is_standby: print_stats.state == PrintState::Standby,
                is_printing: print_stats.state == PrintState::Printing,
                is_paused: print_stats.state == PrintState::Paused,
                is_complete: print_stats.state == PrintState::Complete,
                is_error: print_stats.state == PrintState::Error,
                is_cancelled: print_stats.state == PrintState::Cancelled,
            };
            
            let remaining_time_s_percentage = (elapsed_time / self.progress) - elapsed_time;

            let remaining_time = if let Some(estimate) = self.slicer_time_estimate && estimate > 0 {
                remaining_time_s_percentage * self.progress + (estimate as f32) * (1.0 - self.progress)
            }
            else {
                remaining_time_s_percentage
            };

            self.ui_weak.upgrade_in_event_loop(move |ui| {
                ui.global::<PrintStatus>().set_total_layers(total_layers);
                ui.global::<PrintStatus>().set_current_layer(current_layer);
                ui.global::<PrintStatus>().set_elapsed_time(elapsed_time);
                ui.global::<PrintStatus>().set_estimated_time(remaining_time);
                ui.global::<PrintStatus>().set_filename(filename);
                ui.global::<PrintStatus>().set_state(state);
            })?;
        }

        Ok(())
    }
}
