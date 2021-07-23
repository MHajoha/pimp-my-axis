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
use std::path::{Path, PathBuf};
use std::str::FromStr;

use log::info;
use serde::Deserialize;

use crate::expr::AxisExpression;

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RealDeviceMatcher {
    Path(PathBuf),
    VendorAndProduct { vendor_id: u16, product_id: u16 },
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct VirtDeviceConfig {
    #[serde(default = "default_virt_name")]
    pub name: String,
    #[serde(default = "default_vendor_id")]
    pub vendor_id: u16,
    #[serde(default = "default_product_id")]
    pub product_id: u16,
    pub axes: HashMap<Axis, AxisConfig>,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct AxisConfig {
    pub min: i32,
    pub max: i32,
    pub expr: AxisExpression,
}

fn default_virt_name() -> String {
    return "Pimp-My-Axis Device".to_owned();
}

fn default_vendor_id() -> u16 {
    return 0x1209;
}

fn default_product_id() -> u16 {
    return 0x0001;
}

#[derive(Deserialize, Eq, PartialEq, Hash, Debug, Clone, Copy)]
pub enum Axis {
    X,
    Y,
    Z,
    RX,
    RY,
    RZ,
    Throttle,
    Rudder,
    Wheel,
    Gas,
    Brake,
}

impl FromStr for Axis {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        return match s {
            "X" => Ok(Axis::X),
            "Y" => Ok(Axis::Y),
            "Z" => Ok(Axis::Z),
            "RX" => Ok(Axis::RX),
            "RY" => Ok(Axis::RY),
            "RZ" => Ok(Axis::RZ),
            "Throttle" => Ok(Axis::Throttle),
            "Rudder" => Ok(Axis::Rudder),
            "Wheel" => Ok(Axis::Wheel),
            "Gas" => Ok(Axis::Gas),
            "Brake" => Ok(Axis::Brake),
            _ => Err(format!("Unknown axis name: '{}'", s)),
        };
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub real_devices: HashMap<String, RealDeviceMatcher>,
    pub virt_devices: HashMap<String, VirtDeviceConfig>,
}

pub fn parse_config(path: &Path) -> Config {
    info!("Reading config file '{}'", path.to_string_lossy());
    let file = File::open(path).unwrap();
    return serde_yaml::from_reader(file).unwrap();
}
