use crate::{
    connector::{read_deserialize::OptionalPrinterEvent, websocket_read::PrinterEvent},
    printer_objects::*,
};

// TODO: Fill cache with configuration for min/max temp
#[derive(Debug, Default, Clone)]
pub struct Cache {
    pub webhooks: Webhooks,
    pub motion_report: MotionReport,
    pub gcode_move: GcodeMove,
    pub toolhead: Toolhead,
    pub extruder: Extruder,
    pub heater_bed: HeaterBed,
    pub fan: Fan,
    pub idle_timeout: IdleTimeout,
    pub virtual_sdcard: VirtualSdcard,
    pub print_stats: PrintStats,
    pub display_status: DisplayStatus,
    pub temperature_sensors: Vec<NamedTemperatureSensor>, 
    pub temperature_fans: Vec<NamedTemperatureFan>,
    pub filament_switch_sensors: Vec<NamedFilamentSwitchSensor>,
    pub output_pins: Vec<NamedOutputPin>,
    pub exclude_object: ExcludeObject,
}

impl Cache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn complete_event(&mut self, event: OptionalPrinterEvent) -> PrinterEvent {
        match event {
            OptionalPrinterEvent::Webhooks(webhooks) => {
                self.webhooks.overlay(webhooks);
                PrinterEvent::Webhooks(self.webhooks.clone())
            }
            OptionalPrinterEvent::MotionReport(motion_report) => {
                self.motion_report.overlay(motion_report);
                PrinterEvent::MotionReport(self.motion_report.clone())
            }
            OptionalPrinterEvent::GcodeMove(gcode_move) => {
                self.gcode_move.overlay(gcode_move);
                PrinterEvent::GcodeMove(self.gcode_move.clone())
            }
            OptionalPrinterEvent::Toolhead(toolhead) => {
                self.toolhead.overlay(toolhead);
                PrinterEvent::Toolhead(self.toolhead.clone())
            }
            OptionalPrinterEvent::Extruder(extruder) => {
                self.extruder.overlay(extruder);
                PrinterEvent::Extruder(self.extruder.clone())
            }
            OptionalPrinterEvent::HeaterBed(heater_bed) => {
                self.heater_bed.overlay(heater_bed);
                PrinterEvent::HeaterBed(self.heater_bed.clone())
            }
            OptionalPrinterEvent::Fan(fan) => {
                self.fan.overlay(fan);
                PrinterEvent::Fan(self.fan.clone())
            }
            OptionalPrinterEvent::IdleTimeout(idle_timeout) => {
                self.idle_timeout.overlay(idle_timeout);
                PrinterEvent::IdleTimeout(self.idle_timeout.clone())
            }
            OptionalPrinterEvent::VirtualSdcard(virtual_sdcard) => {
                self.virtual_sdcard.overlay(virtual_sdcard);
                PrinterEvent::VirtualSdcard(self.virtual_sdcard.clone())
            }
            OptionalPrinterEvent::PrintStats(print_stats) => {
                self.print_stats.overlay(print_stats);
                PrinterEvent::PrintStats(self.print_stats.clone())
            }
            OptionalPrinterEvent::DisplayStatus(display_status) => {
                self.display_status.overlay(display_status);
                PrinterEvent::DisplayStatus(self.display_status.clone())
            }
            OptionalPrinterEvent::TemperatureSensor(named_sensor) => {
                let index = self
                    .temperature_sensors
                    .iter()
                    .position(|sensor| sensor.name == named_sensor.name);

                let sensor = match index {
                    Some(index) => {
                        self.temperature_sensors[index]
                            .sensor
                            .overlay(named_sensor.sensor);
                        self.temperature_sensors[index].clone()
                    }
                    None => {
                        let mut new_sensor = TemperatureSensor::default();
                        new_sensor.overlay(named_sensor.sensor);
                        let new_named_sensor = NamedTemperatureSensor {
                            name: named_sensor.name,
                            sensor: new_sensor,
                        };
                        self.temperature_sensors.push(new_named_sensor.clone());
                        new_named_sensor
                    }
                };

                PrinterEvent::TemperatureSensor(sensor)
            }
            OptionalPrinterEvent::TemperatureFan(named_fan) => {
                let index = self
                    .temperature_fans
                    .iter()
                    .position(|fan| fan.name == named_fan.name);

                let fan = match index {
                    Some(index) => {
                        self.temperature_fans[index].fan.overlay(named_fan.fan);
                        self.temperature_fans[index].clone()
                    }
                    None => {
                        let mut new_fan = TemperatureFan::default();
                        new_fan.overlay(named_fan.fan);
                        let new_named_fan = NamedTemperatureFan {
                            name: named_fan.name,
                            fan: new_fan,
                        };
                        self.temperature_fans.push(new_named_fan.clone());
                        new_named_fan
                    }
                };

                PrinterEvent::TemperatureFan(fan)
            }
            OptionalPrinterEvent::FilamentSwitchSensor(named_sensor) => {
                let index = self
                    .filament_switch_sensors
                    .iter()
                    .position(|sensor| sensor.name == named_sensor.name);

                let sensor = match index {
                    Some(index) => {
                        self.filament_switch_sensors[index]
                            .sensor
                            .overlay(named_sensor.sensor);
                        self.filament_switch_sensors[index].clone()
                    }
                    None => {
                        let mut new_sensor = FilamentSwitchSensor::default();
                        new_sensor.overlay(named_sensor.sensor);
                        let new_named_sensor = NamedFilamentSwitchSensor {
                            name: named_sensor.name,
                            sensor: new_sensor,
                        };
                        self.filament_switch_sensors.push(new_named_sensor.clone());
                        new_named_sensor
                    }
                };

                PrinterEvent::FilamentSwitchSensor(sensor)
            }
            OptionalPrinterEvent::OutputPin(named_pin) => {
                let index = self
                    .output_pins
                    .iter()
                    .position(|pin| pin.name == named_pin.name);

                let pin = match index {
                    Some(index) => {
                        self.output_pins[index].pin.overlay(named_pin.pin);
                        self.output_pins[index].clone()
                    }
                    None => {
                        let mut new_pin = OutputPin::default();
                        new_pin.overlay(named_pin.pin);
                        let new_named_pin = NamedOutputPin {
                            name: named_pin.name,
                            pin: new_pin,
                        };
                        self.output_pins.push(new_named_pin.clone());
                        new_named_pin
                    }
                };

                PrinterEvent::OutputPin(pin)
            }
            OptionalPrinterEvent::ExcludeObject(exclude_object) => {
                self.exclude_object.overlay(exclude_object);
                PrinterEvent::ExcludeObject(self.exclude_object.clone())
            }
        }
    }
}
