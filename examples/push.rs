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
extern crate tokio;
extern crate tokio_timer;
extern crate tokio_zmq;
extern crate zmq;

use std::io;
use std::sync::Arc;
use std::time::Duration;
use std::convert::TryInto;

use futures::{FutureExt, StreamExt};
use futures::stream::iter_ok;
use tokio_timer::{Timer, TimerError};
use tokio_zmq::prelude::*;
use tokio_zmq::{Error as ZmqFutError, Push, Socket};

#[derive(Debug)]
enum Error {
    ZmqFut(ZmqFutError),
    Zmq(zmq::Error),
    Io(io::Error),
    Timer(TimerError),
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

impl From<TimerError> for Error {
    fn from(e: TimerError) -> Self {
        Error::Timer(e)
    }
}

fn main() {
    let ctx = Arc::new(zmq::Context::new());
    let workers: Push = Socket::builder(Arc::clone(&ctx))
        .bind("tcp://*:5557")
        .try_into()
        .unwrap();
    let sink: Push = Socket::builder(Arc::clone(&ctx))
        .connect("tcp://localhost:5558")
        .try_into()
        .unwrap();
    let sink2: Push = Socket::builder(ctx)
        .connect("tcp://localhost:5558")
        .try_into()
        .unwrap();

    let start_msg = zmq::Message::from_slice(b"START").unwrap().into();
    let stop_msg = zmq::Message::from_slice(b"STOP").unwrap().into();

    let interval = Timer::default().interval(Duration::from_millis(200));

    let process = sink.send(start_msg).map_err(Error::from).and_then(|_| {
        iter_ok(0..10)
            .zip(interval)
            .map_err(Error::from)
            .and_then(|(i, _)| {
                println!("Sending: {}", i);

                let msg = format!("{}", i);
                let msg = msg.as_bytes();
                let msg = zmq::Message::from_slice(msg)?;

                Ok(msg.into())
            })
            .forward(workers.sink())
            .and_then(move |_| sink2.send(stop_msg).map_err(Error::from))
    });

    tokio::runtime::run2(process.map(|_| ()).or_else(|e| {
        println!("Error: {:?}", e);
        Ok(())
    }));
}
