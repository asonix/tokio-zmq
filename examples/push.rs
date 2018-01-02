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

use futures::{Future, Stream};
use futures::stream::iter_ok;
use tokio_core::reactor::{Core, Interval};
use tokio_zmq::prelude::*;
use tokio_zmq::{Error as ZmqFutError, Push, Socket};

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
    let workers: Push = Socket::new(ctx.clone(), handle.clone())
        .bind("tcp://*:5557")
        .try_into()
        .unwrap();
    let sink: Push = Socket::new(ctx, handle.clone())
        .connect("tcp://localhost:5558")
        .try_into()
        .unwrap();

    let start_msg = {
        let msg = zmq::Message::from_slice(b"START").unwrap();

        let mut multipart = VecDeque::new();
        multipart.push_back(msg);

        multipart
    };
    let stop_msg = {
        let msg = zmq::Message::from_slice(b"STOP").unwrap();

        let mut multipart = VecDeque::new();
        multipart.push_back(msg);

        multipart
    };

    let interval = Interval::new(Duration::from_millis(200), &core.handle()).unwrap();

    let process = sink.send(start_msg).map_err(Error::from).and_then(|_| {
        iter_ok(0..10)
            .zip(interval)
            .map_err(Error::from)
            .and_then(|(i, _)| {
                println!("Sending: {}", i);

                let msg = format!("{}", i);
                let msg = msg.as_bytes();
                let msg = zmq::Message::from_slice(msg)?;

                let mut multipart = VecDeque::new();
                multipart.push_back(msg);

                Ok(multipart)
            })
            .forward(workers.sink::<Error>())
            .and_then(move |_| sink.send(stop_msg).map_err(Error::from))
    });

    core.run(process).unwrap();
}
