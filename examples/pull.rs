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
extern crate tokio_executor;
extern crate tokio_zmq;
extern crate zmq;

use std::sync::Arc;
use std::convert::TryInto;

use futures::{FutureExt, IntoFuture, StreamExt};
use futures::future::Either;
use tokio_zmq::prelude::*;
use tokio_zmq::{Pub, Pull, Sub};
use tokio_zmq::{Multipart, Socket};
use tokio_zmq::Error;

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
    let conn: Pull = Socket::builder(Arc::clone(&ctx))
        .bind("tcp://*:5558")
        .try_into()
        .unwrap();

    let process = conn.stream()
        .controlled(cmd, Stop)
        .map(move |multipart| {
            for msg in multipart {
                if let Some(msg) = msg.as_str() {
                    println!("msg: '{}'", msg);

                    if msg == "STOP" {
                        let send_cmd: Pub = Socket::builder(ctx.clone())
                            .bind("tcp://*:5559")
                            .try_into()
                            .unwrap();
                        return Either::Right(
                            send_cmd
                                .send(zmq::Message::from_slice(b"").unwrap().into())
                                .map(|_| ()),
                        );
                    }
                }
            }

            Either::Left((Ok(()) as Result<(), Error>).into_future())
        })
        .for_each(|_| Ok(()));

    tokio::runtime::run2(process.map(|_| ()).or_else(|e| {
        println!("Error: {:?}", e);
        Ok(())
    }));
}
