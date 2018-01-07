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

use futures::Stream;
use tokio_core::reactor::Core;
use tokio_zmq::prelude::*;
use tokio_zmq::{Socket, Sub};

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let ctx = Rc::new(zmq::Context::new());
    let sub: Sub = Socket::create(ctx, &handle)
        .connect("tcp://localhost:5556")
        .filter(b"")
        .try_into()
        .unwrap();

    let consumer = sub.stream().for_each(|multipart| {
        for msg in multipart {
            if let Some(msg) = msg.as_str() {
                println!("Received: {}", msg);
            }
        }

        Ok(())
    });

    core.run(consumer).unwrap();
}
