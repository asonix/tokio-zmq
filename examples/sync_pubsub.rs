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

extern crate futures;
extern crate tokio_core;
extern crate zmq;
extern crate tokio_zmq;
extern crate log;
extern crate env_logger;

use std::rc::Rc;
use std::convert::TryInto;
use std::thread;
use std::time::Duration;

use futures::stream::iter_ok;
use futures::{Future, Stream};
use tokio_core::reactor::Core;
use tokio_zmq::prelude::*;
use tokio_zmq::{Rep, Req, Pub, Sub};
use tokio_zmq::{Socket, Multipart, Error};

// On my quad-core i7, if I run with too many threads, the context switching takes too long and
// some messages get dropped. 2 subscribers can properly retrieve 1 million messages each, though.
//
// For the nice results, lower the messages and increas the subscribers.
const SUBSCRIBERS: usize = 10;
const MESSAGES: usize = 1_000;

struct Stop;

impl EndHandler for Stop {
    fn should_stop(&mut self, item: &Multipart) -> bool {
        if let Some(msg) = item.get(0) {
            if let Some(msg) = msg.as_str() {
                if msg == "END" {
                    return true;
                }
            }
        }

        false
    }
}

fn publisher_thread() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let ctx = Rc::new(zmq::Context::new());

    let publisher: Pub = Socket::create(Rc::clone(&ctx), &handle)
        .bind("tcp://*:5561")
        .try_into()
        .unwrap();

    let syncservice: Rep = Socket::create(ctx, &handle)
        .bind("tcp://*:5562")
        .try_into()
        .unwrap();

    println!("Waiting for subscribers");

    let runner = iter_ok(0..SUBSCRIBERS)
        .zip(syncservice.stream())
        .map(|(_, _)| {
            zmq::Message::from_slice(b"").unwrap().into()
        })
        .forward(syncservice.sink::<Error>())
        .and_then(|_| {
            println!("Broadcasting message");

            iter_ok(0..MESSAGES)
                .map(|_| {
                    zmq::Message::from_slice(b"Rhubarb").unwrap().into()
                })
                .forward(publisher.sink::<Error>())
        })
        .and_then(|_| {
            let msg = zmq::Message::from_slice(b"END").unwrap();

            publisher.send(msg.into())
        });

    core.run(runner).unwrap();
}

fn subscriber_thread() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let ctx = Rc::new(zmq::Context::new());

    let subscriber: Sub = Socket::create(Rc::clone(&ctx), &handle)
        .connect("tcp://localhost:5561")
        .filter(b"")
        .try_into()
        .unwrap();

    let syncclient: Req = Socket::create(ctx, &handle)
        .connect("tcp://localhost:5562")
        .try_into()
        .unwrap();

    let msg = zmq::Message::from_slice(b"").unwrap();

    let recv = syncclient.recv();

    let runner = syncclient
        .send(msg.into())
        .and_then(move |_| recv)
        .and_then(|_| {
            subscriber
                .stream_with_end(Stop)
                .fold(0, |counter, _| Ok(counter + 1) as Result<usize, Error>)
                .and_then(|total| {
                    println!("Received {} updates", total);
                    Ok(())
                })
        });

    core.run(runner).unwrap();
}

fn main() {
    let mut threads = Vec::new();
    threads.push(thread::spawn(publisher_thread));

    for _ in 0..SUBSCRIBERS {
        thread::sleep(Duration::from_millis(400));
        threads.push(thread::spawn(subscriber_thread));
    }

    for thread in threads {
        thread.join().unwrap();
    }
}
