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

extern crate evdev_rs;
extern crate uinput;

use std::{iter, thread};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, mpsc, RwLock};
use std::thread::JoinHandle;

use clap::Clap;
use log::{debug, trace};

use crate::config::{Axis, Config, parse_config};
use crate::listener::{AxisUpdate, listener_thread_main};
use crate::real::{get_event_devices, RealAxis, RealDevice};
use crate::virt::{VirtAxis, VirtDevice};

mod config;
mod expr;
mod listener;
mod real;
mod virt;

#[derive(Clap)]
struct Opts {
    #[clap(short, long)]
    config: Option<PathBuf>,
}

type AnyIterator<'a, T> = dyn Iterator<Item = T> + 'a;

fn virt_axes(config: &Config) -> Result<HashMap<(String, Axis), VirtAxis>, String> {
    return config
        .virt_devices
        .iter()
        .flat_map(|(name, dev_config)| {
            type ReturnIter<'a> = Box<AnyIterator<'a, Result<((String, Axis), VirtAxis), String>>>;
            let device = match VirtDevice::new(name.clone(), dev_config) {
                Ok(dev) => Rc::new(RefCell::new(dev)),
                Err(err) => return Box::new(iter::once(Err(err))) as ReturnIter,
            };

            return Box::new(dev_config.axes.iter().map(move |(axis, axis_config)| {
                let virt_axis = VirtAxis::new(Rc::clone(&device), *axis, axis_config.clone());
                return Ok(((name.clone(), *axis), virt_axis));
            })) as ReturnIter;
        })
        .collect::<Result<HashMap<(String, Axis), VirtAxis>, String>>();
}

fn real_devices(config: &Config) -> Result<HashMap<String, Arc<RwLock<RealDevice>>>, String> {
    let available_devices = get_event_devices();

    return config
        .real_devices
        .iter()
        .map(|(name, matcher)| {
            Ok((
                name.clone(),
                Arc::new(RwLock::new(RealDevice::new(
                    name.clone(),
                    matcher,
                    &available_devices,
                )?)),
            ))
        })
        .collect::<Result<HashMap<String, Arc<RwLock<RealDevice>>>, String>>();
}

fn real_axes(
    real_devs: &HashMap<String, Arc<RwLock<RealDevice>>>,
    virt_axes: &HashMap<(String, Axis), VirtAxis>,
) -> Result<HashMap<(String, Axis), RealAxis>, String> {
    let mut result = HashMap::new();
    for virt_axis in virt_axes.values() {
        for (dep_dev, dep_axis) in virt_axis.config.expr.dependencies() {
            if let Some(device) = real_devs.get(&dep_dev) {
                if !device.read().unwrap().supports(&dep_axis) {
                    return Err(format!(
                        "Device '{}' does not support axis '{:?}'",
                        dep_dev, dep_axis
                    ));
                }

                let real_axis = result
                    .entry((dep_dev.clone(), dep_axis))
                    .or_insert_with(|| RealAxis::new(Arc::clone(device), dep_axis));
                if !real_axis.downstream.contains(virt_axis) {
                    real_axis.downstream.push(virt_axis.clone());
                    trace!("{}.{:?} -> {}", dep_dev, dep_axis, virt_axis)
                }
            } else {
                return Err(format!(
                    "Expression references device '{}' which is not defined",
                    dep_dev
                ));
            }
        }
    }
    return Ok(result);
}

fn main() {
    env_logger::init();

    let opts: Opts = Opts::parse();
    let xdg = xdg::BaseDirectories::with_prefix("pimp-my-axis").unwrap();
    let system_config_path = Path::new("/etc/pimp-my-axis/config.yml");

    let config_path = if let Some(path) = opts.config {
        path
    } else if let Some(path) = xdg.find_config_file("config.yml") {
        path
    } else if system_config_path.exists() {
        system_config_path.to_path_buf()
    } else {
        panic!("Found no config file")
    };

    let config = parse_config(config_path.as_ref());
    debug!("Config: {:?}", config);

    let real_devices = real_devices(&config).unwrap();

    let virt_axes = virt_axes(&config).unwrap();

    let real_axes = real_axes(&real_devices, &virt_axes).unwrap();

    let (tx, rx) = mpsc::channel::<AxisUpdate>();

    #[allow(unused)]
    let listener_threads: Vec<JoinHandle<()>> = real_devices
        .iter()
        .map(|(_, dev)| {
            let dev_clone = Arc::clone(dev);
            let tx_clone = tx.clone();
            thread::spawn(move || listener_thread_main(dev_clone, tx_clone))
        })
        .collect();

    for update in rx {
        trace!("Received update {:?} from listener thread.", update);

        let real_axis = match real_axes.get(&(update.device.clone(), update.axis)) {
            Some(axis) => axis,
            None => {
                debug!(
                    "Ignoring update for axis {}:{:?} which is not used",
                    update.device, update.axis
                );
                continue;
            }
        };

        let mut axis_values = HashMap::<(String, Axis), i32>::new();
        axis_values.insert((update.device, update.axis), update.new_value);

        for downstream in &real_axis.downstream {
            for (dep_dev, dep_axis) in downstream.config.expr.dependencies() {
                axis_values
                    .entry((dep_dev.clone(), dep_axis))
                    .or_insert_with(|| match real_devices.get(&dep_dev) {
                        Some(device) => device.read().unwrap().read(&dep_axis).unwrap(),
                        None => panic!(
                            "Virtual axis {} references real device {} which does not exist",
                            downstream, dep_dev
                        ),
                    });
            }

            let new_value = downstream.config.expr.eval(&axis_values).unwrap();
            debug!(
                "Calculated new value {} for virtual axis {}",
                new_value, downstream
            );
            downstream
                .device
                .borrow_mut()
                .write(&downstream.axis, new_value)
                .unwrap();
        }
    }
}
