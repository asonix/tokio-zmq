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
extern crate zmq;
extern crate tokio_zmq;

use std::rc::Rc;
use std::convert::TryInto;
use std::collections::VecDeque;

use futures::Stream;
use tokio_core::reactor::Core;
use tokio_zmq::prelude::*;
use tokio_zmq::{Pull, Push, Sub};
use tokio_zmq::{Error as ZmqFutError, Socket};

pub struct Stop;

impl ControlHandler for Stop {
    fn should_stop(&mut self, _: VecDeque<zmq::Message>) -> bool {
        println!("Got stop signal");
        true
    }
}

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let ctx = Rc::new(zmq::Context::new());
    let cmd: Sub = Socket::create(Rc::clone(&ctx), &handle)
        .connect("tcp://localhost:5559")
        .filter(b"")
        .try_into()
        .unwrap();
    let stream: Pull = Socket::create(Rc::clone(&ctx), &handle)
        .connect("tcp://localhost:5557")
        .try_into()
        .unwrap();
    let stream = stream.controlled(cmd);
    let sink: Push = Socket::create(ctx, &handle)
        .connect("tcp://localhost:5558")
        .try_into()
        .unwrap();

    let runner = stream
        .stream(Stop)
        .map(|multipart| {
            for msg in &multipart {
                if let Some(msg) = msg.as_str() {
                    println!("Relaying: {}", msg);
                }
            }
            multipart
        })
        .forward(sink.sink::<ZmqFutError>());

    core.run(runner).unwrap();
}
