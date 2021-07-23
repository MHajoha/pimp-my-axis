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

use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

use log::info;
use uinput::Event;
use uinput::event::absolute::{Position, Wheel};
use uinput::event::Absolute;

use crate::config::{Axis, AxisConfig, VirtDeviceConfig};

pub struct VirtDevice {
    pub name: String,
    pub config: VirtDeviceConfig,
    uinput_device: uinput::Device,
}

fn axis_to_event(axis: &Axis) -> Event {
    return match axis {
        Axis::X => Event::Absolute(Absolute::Position(Position::X)),
        Axis::Y => Event::Absolute(Absolute::Position(Position::Y)),
        Axis::Z => Event::Absolute(Absolute::Position(Position::Z)),
        Axis::RX => Event::Absolute(Absolute::Position(Position::RX)),
        Axis::RY => Event::Absolute(Absolute::Position(Position::RY)),
        Axis::RZ => Event::Absolute(Absolute::Position(Position::RZ)),
        Axis::Throttle => Event::Absolute(Absolute::Wheel(Wheel::Throttle)),
        Axis::Rudder => Event::Absolute(Absolute::Wheel(Wheel::Rudder)),
        Axis::Wheel => Event::Absolute(Absolute::Wheel(Wheel::Position)),
        Axis::Gas => Event::Absolute(Absolute::Wheel(Wheel::Gas)),
        Axis::Brake => Event::Absolute(Absolute::Wheel(Wheel::Brake)),
    };
}

impl VirtDevice {
    pub fn new(name: String, config: &VirtDeviceConfig) -> Result<VirtDevice, String> {
        let mut builder = uinput::default()
            .unwrap()
            .name(&config.name)
            .unwrap()
            .vendor(config.vendor_id)
            .product(config.product_id);

        for (axis, axis_config) in &config.axes {
            builder = builder
                .event(axis_to_event(axis))
                .unwrap()
                .min(axis_config.min)
                .max(axis_config.max);
        }

        let uinput_device = builder.create().map_err(|err| err.to_string())?;

        info!("Created uinput virtual device '{}'", config.name);

        return Ok(VirtDevice {
            name,
            uinput_device,
            config: config.clone(),
        });
    }

    pub fn write(&mut self, axis: &Axis, value: i32) -> Result<(), String> {
        self.uinput_device
            .send(axis_to_event(axis), value)
            .map_err(|err| err.to_string())?;
        self.uinput_device
            .synchronize()
            .map_err(|err| err.to_string())
    }
}

#[derive(Clone)]
pub struct VirtAxis {
    pub device: Rc<RefCell<VirtDevice>>,
    pub axis: Axis,
    pub config: AxisConfig,
}

impl VirtAxis {
    pub fn new(device: Rc<RefCell<VirtDevice>>, axis: Axis, config: AxisConfig) -> VirtAxis {
        return VirtAxis {
            device,
            axis,
            config,
        };
    }
}

impl PartialEq for VirtAxis {
    fn eq(&self, other: &Self) -> bool {
        return self.axis == other.axis
            && self.config.expr == other.config.expr
            && Rc::ptr_eq(&self.device, &other.device);
    }
}

impl Eq for VirtAxis {}

impl Display for VirtAxis {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}:{:?}",
            self.device.borrow().name,
            self.axis
        ))
    }
}
