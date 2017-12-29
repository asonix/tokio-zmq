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

use futures::{Future, Stream};
use tokio_core::reactor::Core;
use zmq_futures::{Controlled, Pub, Pull, SinkSocket, Sub};
use zmq_futures::async::ControlHandler;

pub struct Stop;

impl ControlHandler for Stop {
    type Request = Vec<zmq::Message>;

    fn should_stop(&self, _: Self::Request) -> bool {
        println!("Got stop signal");
        true
    }
}

fn main() {
    let mut core = Core::new().unwrap();
    let cmd = Sub::new()
        .connect("tcp://localhost:5559")
        .more()
        .filter(b"")
        .unwrap();
    let conn = Pull::controlled(cmd).bind("tcp://*:5558").build().unwrap();
    let send_cmd = Pub::new().bind("tcp://*:5559").build().unwrap();

    let handle = core.handle();

    let process = conn.controlled_stream::<Stop, zmq::Error>(Stop)
        .and_then(|msg| msg.map(|msg| msg.to_vec()).concat2())
        .for_each(move |msg| {
            let msg = String::from_utf8(msg).unwrap();
            println!("msg: '{}'", msg);

            if msg == "STOP" {
                handle.spawn(
                    send_cmd
                        .send(zmq::Message::from_slice(b"").unwrap())
                        .map_err(|_| ()),
                );
            }

            Ok(())
        });

    core.run(process).unwrap();
}
