use std::rc::Rc;

use moonraker_rs::connector::websocket_read::PrinterEvent;
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};

use crate::{
    application_error::ApplicationError, event_loop::EventLoop, AppWindow, Heater, HeaterFan, TemperatureSensor, TemperatureSensors
};

pub trait TemperatureDevicesHandler {
    fn handle_temperature_devices_update(
        &mut self,
        printer_event: &PrinterEvent,
    ) -> Result<(), ApplicationError>;
}

impl TemperatureDevicesHandler for EventLoop {
    fn handle_temperature_devices_update(
        &mut self,
        printer_event: &PrinterEvent,
    ) -> Result<(), ApplicationError> {
        if let PrinterEvent::Extruder(extruder_event) = printer_event {
            let extruder = Heater {
                name: SharedString::from("extruder"),
                target: extruder_event.target as i32,
                temperature: extruder_event.temperature as i32,
                min_temp: extruder_event.temp_control.min_temp,
                max_temp: extruder_event.temp_control.max_temp,
                step: extruder_event.temp_control.step_temp,
            };

            self.ui_weak
                .upgrade_in_event_loop(Box::new(move |ui: AppWindow| {
                    ui.global::<TemperatureSensors>().set_extruder(extruder)
                }))?;
        }

        if let PrinterEvent::HeaterBed(heater_bed_event) = printer_event {
            let bed = Heater {
                name: SharedString::from("heater_bed"),
                target: heater_bed_event.target as i32,
                temperature: heater_bed_event.temperature as i32,
                min_temp: heater_bed_event.temp_control.min_temp,
                max_temp: heater_bed_event.temp_control.max_temp,
                step: heater_bed_event.temp_control.step_temp,
            };

            self.ui_weak.upgrade_in_event_loop(move |ui: AppWindow| {
                ui.global::<TemperatureSensors>().set_heated_bed(bed)
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

                    let mut entries = match &current_sensors {
                        Some(model) => model.iter().collect::<Vec<TemperatureSensor>>(),
                        None => vec![],
                    };

                    let index = entries
                        .iter()
                        .position(|sensor| sensor.name == sensor_event.name);

                    match index {
                        Some(index) => entries[index].temperature = sensor_event.temperature as i32,
                        None => entries.push(sensor_event),
                    }

                    ui.global::<TemperatureSensors>()
                        .set_temperature_sensors(ModelRc::new(Rc::new(VecModel::from(entries))));
                })?;
        }

        if let PrinterEvent::TemperatureFan(temperature_fan_event) = printer_event {
            let sensor_event = HeaterFan {
                heater: Heater {
                    name: SharedString::from(&temperature_fan_event.name),
                    temperature: temperature_fan_event.fan.temperature as i32,
                    target: temperature_fan_event.fan.target as i32,
                    min_temp: temperature_fan_event.fan.temp_control.min_temp,
                    max_temp: temperature_fan_event.fan.temp_control.max_temp,
                    step: temperature_fan_event.fan.temp_control.step_temp,
                },
                speed: temperature_fan_event.fan.speed,
            };

            self.ui_weak
                .upgrade_in_event_loop(move |ui| {
                    let heater_fans =
                        ui.global::<TemperatureSensors>().get_heater_fans();
                    let current_sensors = heater_fans
                        .as_any()
                        .downcast_ref::<VecModel<HeaterFan>>();

                    let mut entries = match &current_sensors {
                        Some(model) => model.iter().collect::<Vec<HeaterFan>>(),
                        None => vec![],
                    };

                    let index = entries
                        .iter()
                        .position(|sensor| sensor.heater.name == sensor_event.heater.name);

                    match index {
                        Some(index) => {
                            entries[index].heater.temperature = sensor_event.heater.temperature;
                            entries[index].heater.target = sensor_event.heater.target;
                            entries[index].speed = sensor_event.speed;
                        },
                        None => entries.push(sensor_event),
                    }

                    ui.global::<TemperatureSensors>()
                        .set_heater_fans(ModelRc::new(Rc::new(VecModel::from(entries))));
                })?;
        }

        Ok(())
    }
}
