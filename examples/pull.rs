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

#![feature(try_from)]

extern crate futures;
extern crate tokio_core;
extern crate tokio_zmq;
extern crate zmq;

use std::rc::Rc;
use std::convert::TryInto;

use futures::{Future, Stream};
use tokio_core::reactor::Core;
use tokio_zmq::prelude::*;
use tokio_zmq::{Pub, Pull, Sub};
use tokio_zmq::{Multipart, Socket};

pub struct Stop;

impl ControlHandler for Stop {
    fn should_stop(&mut self, _: Multipart) -> bool {
        println!("Got stop signal");
        true
    }
}

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let ctx = Rc::new(zmq::Context::new());
    let cmd: Sub = Socket::builder(Rc::clone(&ctx), &handle)
        .connect("tcp://localhost:5559")
        .filter(b"")
        .try_into()
        .unwrap();
    let conn: Pull = Socket::builder(Rc::clone(&ctx), &handle)
        .bind("tcp://*:5558")
        .try_into()
        .unwrap();
    let send_cmd: Pub = Socket::builder(ctx, &handle)
        .bind("tcp://*:5559")
        .try_into()
        .unwrap();

    let process = conn.stream()
        .controlled(cmd, Stop)
        .for_each(move |multipart| {
            for msg in multipart {
                if let Some(msg) = msg.as_str() {
                    println!("msg: '{}'", msg);

                    if msg == "STOP" {
                        handle.spawn(
                            send_cmd
                                .send(zmq::Message::from_slice(b"").unwrap().into())
                                .map_err(|_| ()),
                        );
                    }
                }
            }

            Ok(())
        });

    core.run(process).unwrap();
}
