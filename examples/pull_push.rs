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
use zmq_futures::push::Push;
use zmq_futures::pull::Pull;
use zmq_futures::{SinkSocket, StreamSocket};

#[derive(Debug)]
enum Error {
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

fn main() {
    let mut core = Core::new().unwrap();
    let stream = Pull::new().connect("tcp://localhost:5557").build().unwrap();
    let sink = Push::new().connect("tcp://localhost:5558").build().unwrap();

    let process = stream
        .stream()
        .map_err(Error::from)
        .and_then(|msg| {
            msg.map(|msg| msg.to_vec()).concat2().map_err(Error::from)
        })
        .and_then(|msg| {
            let msg = String::from_utf8(msg)?;
            println!("msg: '{}'", msg);
            zmq::Message::from_slice(msg.as_bytes()).map_err(Error::from)
        })
        .forward(sink.sink::<Error>());

    core.run(process).unwrap();
}
