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

extern crate zmq;
extern crate tokio_core;
extern crate futures;
extern crate zmq_futures;

use tokio_core::reactor::Core;
use futures::stream::iter_ok;
use futures::{Future, Stream};

use zmq_futures::async::req::ReqBuilder;

fn main() {
    let zmq_async = ReqBuilder::new().connect("tcp://localhost:5560").unwrap();

    let mut core = Core::new().unwrap();

    let stream = iter_ok(5..10)
        .and_then(|req| {
            println!("sending: {}", req);
            zmq_async
                .send(zmq::Message::from_slice("Hello".as_bytes()).unwrap())
                .map(move |message| (req, message))
        })
        .for_each(|(req, message)| {
            println!("Received reply {} {}", req, message.as_str().unwrap());
            Ok(())
        });

    core.run(stream).unwrap();
}
