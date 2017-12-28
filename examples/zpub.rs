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
use std::time::Duration;

use futures::Stream;
use tokio_core::reactor::{Core, Interval};
use zmq_futures::Pub;
use zmq_futures::SinkSocket;
use zmq_futures::service::response::Singleton;

#[derive(Debug)]
enum Error {
    Zmq(zmq::Error),
    Io(io::Error),
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
    let conn = Pub::new().bind("tcp://*:5556").build().unwrap();

    println!("Got zmq");

    let producer = Interval::new(Duration::from_secs(1), &core.handle())
        .unwrap()
        .map_err(Error::from)
        .and_then(|_| {
            println!("Sending 'Hello'");
            zmq::Message::from_slice(b"Hello")
                .map_err(Error::from)
                .map(|msg| msg.into())
        })
        .forward(conn.sink::<Singleton<Error>, Error>());

    core.run(producer).unwrap();
}
