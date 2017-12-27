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

use futures::Future;
use tokio_core::reactor::Core;
use zmq_futures::rep::{RepBuilder, RepHandler};

#[derive(Debug)]
pub enum Error {
    One,
}

impl From<()> for Error {
    fn from(_: ()) -> Self {
        Error::One
    }
}

#[derive(Clone)]
pub struct Echo;

impl RepHandler for Echo {
    type Request = zmq::Message;
    type Response = zmq::Message;
    type Error = Error;

    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        Box::new(futures::future::ok(req))
    }
}

fn main() {
    let mut core = Core::new().unwrap();
    let zmq = RepBuilder::new()
        .handler(Echo {})
        .bind("tcp://*:5560")
        .unwrap();

    println!("Got zmq");

    core.run(zmq.runner()).unwrap();
}
