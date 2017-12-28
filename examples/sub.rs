/*
 * This file is part of ZeroMQ Futures.
 *
 * Copyright © 2017 Riley Trautman
 *
 * ZeroMQ Futures is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * ZeroMQ Futures is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with ZeroMQ Futures.  If not, see <http://www.gnu.org/licenses/>.
 */

extern crate futures;
extern crate tokio_core;
extern crate zmq;
extern crate zmq_futures;

use futures::Stream;
use tokio_core::reactor::Core;
use zmq_futures::sub::Sub;

fn main() {
    let mut core = Core::new().unwrap();
    let conn = Sub::new()
        .connect("tcp://localhost:5556")
        .filter(b"H")
        .unwrap();

    let consumer = conn.stream()
        .and_then(|msg| msg.map(|msg| msg.to_vec()).concat2())
        .for_each(|msg| {
            let msg = String::from_utf8(msg).unwrap();
            println!("Received: '{}'", msg);

            Ok(())
        });

    core.run(consumer).unwrap();
}
