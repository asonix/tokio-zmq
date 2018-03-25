/*
 * This file is part of Tokio ZMQ.
 *
 * Copyright © 2017 Riley Trautman
 *
 * Tokio ZMQ is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Tokio ZMQ is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Tokio ZMQ.  If not, see <http://www.gnu.org/licenses/>.
 */

#![feature(try_from)]

extern crate futures_util;
extern crate tokio;
extern crate tokio_timer_futures2 as tokio_timer;
extern crate tokio_zmq;
extern crate zmq;

use std::convert::TryInto;
use std::io;
use std::sync::Arc;
use std::time::Duration;

use futures_util::{FutureExt, StreamExt};
use tokio_timer::{Timer, TimerError};
use tokio_zmq::prelude::*;
use tokio_zmq::{Error as ZmqFutError, Pub, Socket};

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
    let zpub: Pub = Socket::builder(ctx)
        .bind("tcp://*:5556")
        .try_into()
        .unwrap();

    let producer = Timer::default()
        .interval(Duration::from_secs(1))
        .map_err(Error::from)
        .and_then(|_| {
            println!("Sending 'Hello'");
            zmq::Message::from_slice(b"Hello")
                .map_err(Error::from)
                .map(|msg| msg.into())
        })
        .forward(zpub.sink());

    tokio::runtime::run2(producer.map(|_| ()).or_else(|e| {
        println!("Error in producer: {:?}", e);
        Ok(())
    }));
}
