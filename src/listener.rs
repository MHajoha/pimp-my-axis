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

use std::iter;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::Sender;

use log::{debug, trace};

use crate::config::Axis;
use crate::real::RealDevice;

pub fn listener_thread_main(device: Arc<RwLock<RealDevice>>, tx: Sender<AxisUpdate>) {
    debug!(
        "Listener thread for device {} started.",
        device.read().unwrap().name
    );

    for update in iter::from_fn(|| device.read().unwrap().next_event()) {
        trace!("Forwarding update {:?}.", update);
        tx.send(update).unwrap();
    }

    debug!(
        "Listener thread for device {} finished.",
        device.read().unwrap().name
    );
}

#[derive(Debug)]
pub struct AxisUpdate {
    pub device: String,
    pub axis: Axis,
    pub new_value: i32,
}
