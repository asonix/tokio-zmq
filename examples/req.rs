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

extern crate env_logger;
extern crate futures_util;
extern crate log;
extern crate tokio;
extern crate tokio_zmq;
extern crate zmq;

use std::sync::Arc;
use std::convert::TryInto;

use futures_util::{FutureExt, StreamExt};
use futures_util::stream::iter_ok;
use tokio_zmq::prelude::*;
use tokio_zmq::{Multipart, Req, Socket};

fn build_multipart(i: usize) -> Multipart {
    let mut multipart = Multipart::new();

    let msg1 = zmq::Message::from_slice(format!("Hewwo? {}", i).as_bytes()).unwrap();
    let msg2 = zmq::Message::from_slice(format!("Mr Obama??? {}", i).as_bytes()).unwrap();

    multipart.push_back(msg1);
    multipart.push_back(msg2);
    multipart
}

fn main() {
    env_logger::init().unwrap();

    let ctx = Arc::new(zmq::Context::new());
    let req: Req = Socket::builder(ctx)
        .connect("tcp://localhost:5560")
        .try_into()
        .unwrap();

    let socket = req.socket();

    let runner = socket.send(build_multipart(0)).and_then(|(sock, file)| {
        let (sink, stream) = Socket::from_sock_and_file(sock, file).sink_stream().split();

        stream
            .zip(iter_ok(1..10_000))
            .map(|(multipart, i)| {
                for msg in multipart {
                    if let Some(msg) = msg.as_str() {
                        println!("Received: {}", msg);
                    }
                }
                build_multipart(i)
            })
            .forward(sink)
    });

    tokio::runtime::run2(runner.map(|_| ()).or_else(|e| {
        println!("Error: {:?}", e);
        Ok(())
    }));
}
