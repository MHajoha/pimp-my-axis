/*
 * This file is part of Pimp-My-Axis.
 *
 * Pimp-My-Axis is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Pimp-My-Axis is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Pimp-My-Axis. If not, see <http://www.gnu.org/licenses/>.
 */

use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use evdev_rs::DeviceWrapper;
use evdev_rs::enums::{EV_ABS, EventCode};
use libudev::Device;
use log::{debug, info, trace};
use nix::libc;

use crate::config::{Axis, RealDeviceMatcher};
use crate::listener::AxisUpdate;
use crate::virt::VirtAxis;

pub struct RealDevice {
    pub name: String,
    pub matcher: RealDeviceMatcher,
    evdev_device: evdev_rs::Device,
}

fn axis_to_event_code(axis: &Axis) -> EventCode {
    return match axis {
        Axis::X => EventCode::EV_ABS(EV_ABS::ABS_X),
        Axis::Y => EventCode::EV_ABS(EV_ABS::ABS_Y),
        Axis::Z => EventCode::EV_ABS(EV_ABS::ABS_Z),
        Axis::RX => EventCode::EV_ABS(EV_ABS::ABS_RX),
        Axis::RY => EventCode::EV_ABS(EV_ABS::ABS_RY),
        Axis::RZ => EventCode::EV_ABS(EV_ABS::ABS_RZ),
        Axis::Throttle => EventCode::EV_ABS(EV_ABS::ABS_THROTTLE),
        Axis::Rudder => EventCode::EV_ABS(EV_ABS::ABS_RUDDER),
        Axis::Wheel => EventCode::EV_ABS(EV_ABS::ABS_WHEEL),
        Axis::Gas => EventCode::EV_ABS(EV_ABS::ABS_GAS),
        Axis::Brake => EventCode::EV_ABS(EV_ABS::ABS_BRAKE),
    };
}

fn event_code_to_axis(event_code: &EventCode) -> Option<Axis> {
    return match event_code {
        EventCode::EV_ABS(EV_ABS::ABS_X) => Some(Axis::X),
        EventCode::EV_ABS(EV_ABS::ABS_Y) => Some(Axis::Y),
        EventCode::EV_ABS(EV_ABS::ABS_Z) => Some(Axis::Z),
        EventCode::EV_ABS(EV_ABS::ABS_RX) => Some(Axis::RX),
        EventCode::EV_ABS(EV_ABS::ABS_RY) => Some(Axis::RY),
        EventCode::EV_ABS(EV_ABS::ABS_RZ) => Some(Axis::RZ),
        EventCode::EV_ABS(EV_ABS::ABS_THROTTLE) => Some(Axis::Throttle),
        EventCode::EV_ABS(EV_ABS::ABS_RUDDER) => Some(Axis::Rudder),
        EventCode::EV_ABS(EV_ABS::ABS_WHEEL) => Some(Axis::Wheel),
        EventCode::EV_ABS(EV_ABS::ABS_GAS) => Some(Axis::Gas),
        EventCode::EV_ABS(EV_ABS::ABS_BRAKE) => Some(Axis::Brake),
        _ => None,
    };
}

pub fn get_event_devices() -> HashMap<(u16, u16), PathBuf> {
    let mut result = HashMap::new();

    let context = libudev::Context::new().unwrap();
    let mut enumerator = libudev::Enumerator::new(&context).unwrap();
    enumerator.match_subsystem("input").unwrap();
    enumerator.match_is_initialized().unwrap();
    for event_device in enumerator.scan_devices().unwrap().filter(is_event_device) {
        let dev_node = match event_device.devnode() {
            Some(value) => value.to_path_buf(),
            None => continue,
        };

        let vendor_id = match event_device.property_value("ID_VENDOR_ID") {
            Some(value) => u16::from_str_radix(&value.to_string_lossy(), 16).unwrap(),
            None => {
                debug!(
                    "Event device '{}' does not have a known vendor ID. Ignoring.",
                    dev_node.to_string_lossy()
                );
                continue;
            }
        };

        let product_id = match event_device.property_value("ID_MODEL_ID") {
            Some(value) => u16::from_str_radix(&value.to_string_lossy(), 16).unwrap(),
            None => {
                debug!(
                    "Event device '{}' does not have a known product ID. Ignoring.",
                    dev_node.to_string_lossy()
                );
                continue;
            }
        };

        debug!(
            "Found event device '{}' with IDs '{:04x}:{:04x}'.",
            dev_node.to_string_lossy(),
            vendor_id,
            product_id
        );

        result.insert((vendor_id, product_id), dev_node);
    }

    return result;
}

fn is_event_device(device: &Device) -> bool {
    return match device.sysname() {
        Some(name) => name.to_string_lossy().starts_with("event"),
        None => false,
    };
}

impl RealDevice {
    pub fn new(
        name: String,
        matcher: &RealDeviceMatcher,
        paths_by_ids: &HashMap<(u16, u16), PathBuf>,
    ) -> Result<RealDevice, String> {
        let path = match matcher {
            RealDeviceMatcher::Path(path) => path.clone(),
            RealDeviceMatcher::VendorAndProduct {
                vendor_id,
                product_id,
            } => match paths_by_ids.get(&(*vendor_id, *product_id)) {
                Some(path) => path.clone(),
                None => {
                    return Err(format!(
                        "No device with IDs '{:04x}:{:04x}' was found",
                        vendor_id, product_id
                    ));
                }
            },
        };

        debug!(
            "Using path '{}' for matcher {:?}",
            path.to_string_lossy(),
            matcher
        );

        let file = File::open(&path).map_err(|err| err.to_string())?;
        let evdev_device = evdev_rs::Device::new_from_file(file).unwrap();

        info!("Opened event device '{}'", path.to_string_lossy());

        return Ok(RealDevice {
            name,
            matcher: matcher.clone(),
            evdev_device,
        });
    }

    pub fn read(&self, axis: &Axis) -> Result<i32, String> {
        return match self.evdev_device.abs_info(&axis_to_event_code(axis)) {
            Some(info) => Ok(info.value),
            None => Err(format!(
                "Device {:?} does not support axis {:?}",
                self.matcher, axis
            )),
        };
    }

    pub fn next_event(&self) -> Option<AxisUpdate> {
        let mut read_flag = evdev_rs::ReadFlag::NORMAL;
        loop {
            match self.evdev_device.next_event(read_flag) {
                Ok((evdev_rs::ReadStatus::Success, event)) => {
                    return Some(AxisUpdate {
                        device: self.name.clone(),
                        axis: match event_code_to_axis(&event.event_code) {
                            Some(axis) => axis,
                            None => {
                                trace!("Unhandled event code: {}", event.event_code);
                                continue;
                            }
                        },
                        new_value: event.value,
                    });
                }
                Ok((evdev_rs::ReadStatus::Sync, _)) => read_flag = evdev_rs::ReadFlag::SYNC,
                Err(err) => match err.raw_os_error() {
                    Some(libc::EAGAIN) => read_flag = evdev_rs::ReadFlag::NORMAL,
                    Some(_) | None => panic!("Unable to get next event: {}", err),
                },
            }
        }
    }

    pub fn supports(&self, axis: &Axis) -> bool {
        return self.evdev_device.has_event_code(&axis_to_event_code(axis));
    }
}

pub struct RealAxis {
    pub device: Arc<RwLock<RealDevice>>,
    pub axis: Axis,
    pub downstream: Vec<VirtAxis>,
}

impl RealAxis {
    pub fn new(device: Arc<RwLock<RealDevice>>, axis: Axis) -> RealAxis {
        return RealAxis {
            device,
            axis,
            downstream: Vec::new(),
        };
    }
}

unsafe impl Send for RealDevice {}

unsafe impl Sync for RealDevice {}
