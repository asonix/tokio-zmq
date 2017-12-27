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

use std::io;
use std::string::FromUtf8Error;

use futures::{Future, Stream};
use tokio_core::reactor::Core;
use zmq_futures::rep::{RepBuilder, RepHandler};
use zmq_futures::async::stream::MsgStream;

#[derive(Debug)]
pub enum Error {
    Zmq(zmq::Error),
    Io(io::Error),
    Utf8(FromUtf8Error),
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

impl From<FromUtf8Error> for Error {
    fn from(e: FromUtf8Error) -> Self {
        Error::Utf8(e)
    }
}

#[derive(Clone)]
pub struct Echo;

impl RepHandler for Echo {
    type Request = MsgStream;
    type Response = zmq::Message;
    type Error = Error;

    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let res = req.map(|msg| msg.to_vec())
            .concat2()
            .map_err(Error::from)
            .and_then(|msg| {
                String::from_utf8(msg)
                    .map_err(Error::from)
                    .map(|msg| {
                        println!("Received: '{}'", msg);
                        msg
                    })
                    .and_then(|msg| {
                        zmq::Message::from_slice(msg.as_bytes()).map_err(Error::from)
                    })
            });

        Box::new(res)
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
