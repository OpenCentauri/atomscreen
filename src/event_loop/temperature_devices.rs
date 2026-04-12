use std::rc::Rc;

use moonraker_rs::connector::websocket_read::PrinterEvent;
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};

use crate::{
    application_error::ApplicationError, event_loop::EventLoop, AppWindow, Heater, HeaterFan, TemperatureSensor, TemperatureSensors
};

impl EventLoop {
    pub fn handle_temperature_devices_update(
        &self,
        printer_event: &PrinterEvent,
    ) -> Result<(), ApplicationError> {
        if let PrinterEvent::Extruder(extruder_event) = printer_event {
            let extruder_event = extruder_event.clone();
            
            self.ui_weak
                .upgrade_in_event_loop(Box::new(move |ui: AppWindow| {
                    ui.global::<TemperatureSensors>().set_extruder(Heater {
                        name: SharedString::from("extruder"),
                        target: extruder_event.target as i32,
                        temperature: extruder_event.temperature as i32,
                        presets: ModelRc::new(Rc::new(VecModel::from(extruder_event.configuration.presets.iter().map(|f| *f as i32).collect::<Vec<i32>>()))),
                    })
                }))?;
        }

        if let PrinterEvent::HeaterBed(heater_bed_event) = printer_event {
            let heater_bed_event = heater_bed_event.clone();

            self.ui_weak.upgrade_in_event_loop(move |ui: AppWindow| {
                ui.global::<TemperatureSensors>().set_heated_bed(Heater {
                    name: SharedString::from("heater_bed"),
                    target: heater_bed_event.target as i32,
                    temperature: heater_bed_event.temperature as i32,
                    presets: ModelRc::new(Rc::new(VecModel::from(heater_bed_event.configuration.presets.iter().map(|f| *f as i32).collect::<Vec<i32>>()))),
                })
            })?;
        }

        if let PrinterEvent::TemperatureSensor(temperature_sensor_event) = printer_event {
            let sensor_event = TemperatureSensor {
                name: SharedString::from(&temperature_sensor_event.name),
                temperature: temperature_sensor_event.sensor.temperature as i32,
            };

            self.ui_weak
                .upgrade_in_event_loop(move |ui| {
                    let temperature_sensors =
                        ui.global::<TemperatureSensors>().get_temperature_sensors();
                    let current_sensors = temperature_sensors
                        .as_any()
                        .downcast_ref::<VecModel<TemperatureSensor>>();

                    match current_sensors {
                        Some(model) => {
                            let index = (0..model.row_count())
                                .find(|&i| model.row_data(i).map_or(false, |s| s.name == sensor_event.name));

                            match index {
                                Some(i) => {
                                    let mut entry = model.row_data(i).unwrap();
                                    entry.temperature = sensor_event.temperature;
                                    model.set_row_data(i, entry);
                                }
                                None => model.push(sensor_event),
                            }
                        }
                        None => {
                            ui.global::<TemperatureSensors>()
                                .set_temperature_sensors(ModelRc::new(Rc::new(VecModel::from(vec![sensor_event]))));
                        }
                    }
                })?;
        }

        if let PrinterEvent::TemperatureFan(temperature_fan_event) = printer_event {
            let temperature_fan_event = temperature_fan_event.clone();


            self.ui_weak
                .upgrade_in_event_loop(move |ui| {
                    let heater_fans =
                        ui.global::<TemperatureSensors>().get_heater_fans();
                    let current_sensors = heater_fans
                        .as_any()
                        .downcast_ref::<VecModel<HeaterFan>>();

                    let sensor_event = HeaterFan {
                        heater: Heater {
                            name: SharedString::from(&temperature_fan_event.name),
                            temperature: temperature_fan_event.fan.temperature as i32,
                            target: temperature_fan_event.fan.target as i32,
                            presets: ModelRc::new(Rc::new(VecModel::from(temperature_fan_event.fan.configuration.presets.iter().map(|f| *f as i32).collect::<Vec<i32>>())))
                        },
                        speed: temperature_fan_event.fan.speed,
                    };

                    match current_sensors {
                        Some(model) => {
                            let index = (0..model.row_count())
                                .find(|&i| model.row_data(i).map_or(false, |s| s.heater.name == sensor_event.heater.name));

                            match index {
                                Some(i) => {
                                    let mut entry = model.row_data(i).unwrap();
                                    entry.heater.temperature = sensor_event.heater.temperature;
                                    entry.heater.target = sensor_event.heater.target;
                                    entry.speed = sensor_event.speed;
                                    model.set_row_data(i, entry);
                                }
                                None => model.push(sensor_event),
                            }
                        }
                        None => {
                            ui.global::<TemperatureSensors>()
                                .set_heater_fans(ModelRc::new(Rc::new(VecModel::from(vec![sensor_event]))));
                        }
                    }
                })?;
        }

        Ok(())
    }
}
