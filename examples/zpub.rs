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

use std::io;
use std::rc::Rc;
use std::time::Duration;
use std::convert::TryInto;
use std::collections::VecDeque;

use futures::Stream;
use tokio_core::reactor::{Core, Interval};
use tokio_zmq::prelude::*;
use tokio_zmq::{Error as ZmqFutError, Pub, Socket};

#[derive(Debug)]
enum Error {
    ZmqFut(ZmqFutError),
    Zmq(zmq::Error),
    Io(io::Error),
}

impl From<ZmqFutError> for Error {
    fn from(e: ZmqFutError) -> Self {
        Error::ZmqFut(e)
    }
}

impl From<zmq::Error> for Error {
    fn from(e: zmq::Error) -> Self {
        Error::Zmq(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let ctx = Rc::new(zmq::Context::new());
    let zpub: Pub = Socket::create(ctx, &handle)
        .bind("tcp://*:5556")
        .try_into()
        .unwrap();

    let producer = Interval::new(Duration::from_secs(1), &handle)
        .unwrap()
        .map_err(Error::from)
        .and_then(|_| {
            println!("Sending 'Hello'");
            zmq::Message::from_slice(b"Hello")
                .map_err(Error::from)
                .map(|msg| {
                    let mut multipart = VecDeque::new();
                    multipart.push_back(msg);
                    multipart
                })
        })
        .forward(zpub.sink::<Error>());

    core.run(producer).unwrap();
}
