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

extern crate env_logger;
extern crate futures;
extern crate log;
extern crate tokio_core;
extern crate tokio_zmq;
extern crate zmq;

use std::rc::Rc;
use std::convert::TryInto;

use futures::{Future, Stream};
use futures::stream::iter_ok;
use tokio_core::reactor::Core;
use tokio_zmq::prelude::*;
use tokio_zmq::{Multipart, Req, Socket};

fn main() {
    env_logger::init().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let ctx = Rc::new(zmq::Context::new());
    let req: Req = Socket::builder(ctx, &handle)
        .connect("tcp://localhost:5560")
        .try_into()
        .unwrap();

    let runner = iter_ok(0..10_000)
        .and_then(|i| {
            let mut multipart = Multipart::new();

            let msg1 = zmq::Message::from_slice(format!("Hewwo? {}", i).as_bytes()).unwrap();
            let msg2 = zmq::Message::from_slice(format!("Mr Obama??? {}", i).as_bytes()).unwrap();

            multipart.push_back(msg1);
            multipart.push_back(msg2);

            let resp = req.recv();
            req.send(multipart).and_then(|_| resp)
        })
        .and_then(|multipart| {
            for msg in multipart {
                if let Some(msg) = msg.as_str() {
                    println!("Received: {}", msg);
                }
            }
            Ok(())
        })
        .for_each(|_| Ok(()));

    core.run(runner).unwrap();
}
