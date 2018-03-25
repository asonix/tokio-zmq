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
extern crate tokio_zmq;
extern crate zmq;

use std::sync::Arc;
use std::convert::TryInto;

use futures_util::{FutureExt, StreamExt};
use tokio_zmq::prelude::*;
use tokio_zmq::{Pull, Push, Sub};
use tokio_zmq::{Multipart, Socket};

pub struct Stop;

impl ControlHandler for Stop {
    fn should_stop(&mut self, _: Multipart) -> bool {
        println!("Got stop signal");
        true
    }
}

fn main() {
    let ctx = Arc::new(zmq::Context::new());
    let cmd: Sub = Socket::builder(Arc::clone(&ctx))
        .connect("tcp://localhost:5559")
        .filter(b"")
        .try_into()
        .unwrap();
    let stream: Pull = Socket::builder(Arc::clone(&ctx))
        .connect("tcp://localhost:5557")
        .try_into()
        .unwrap();
    let sink: Push = Socket::builder(ctx)
        .connect("tcp://localhost:5558")
        .try_into()
        .unwrap();

    let runner = stream
        .stream()
        .controlled(cmd.stream(), Stop)
        .map(|multipart| {
            for msg in &multipart {
                if let Some(msg) = msg.as_str() {
                    println!("Relaying: {}", msg);
                }
            }
            multipart
        })
        .forward(sink.sink());

    tokio::runtime::run2(runner.map(|_| ()).or_else(|e| {
        println!("Error!: {:?}", e);
        Ok(())
    }));
}
