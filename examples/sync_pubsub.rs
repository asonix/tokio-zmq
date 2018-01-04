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
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

use futures::stream::iter_ok;
use futures::{Future, Stream};
use tokio_core::reactor::Core;
use tokio_zmq::prelude::*;
use tokio_zmq::{Rep, Req, Pub, Sub};
use tokio_zmq::{Socket, Error};

const SUBSCRIBERS: usize = 10;

struct Stop;

impl ControlHandler for Stop {
    fn should_stop(&self, _: VecDeque<zmq::Message>) -> bool {
        println!("Got stop signal");
        true
    }
}

fn publisher_thread() {
    let mut core = Core::new().unwrap();
    let ctx = Rc::new(zmq::Context::new());

    let publisher: Pub = Socket::new(Rc::clone(&ctx), core.handle())
        .bind("tcp://*:5561")
        .try_into()
        .unwrap();

    let syncservice: Rep = Socket::new(ctx, core.handle())
        .bind("tcp://*:5562")
        .try_into()
        .unwrap();

    println!("Waiting for subscribers");

    let runner = iter_ok(0..SUBSCRIBERS)
        .zip(syncservice.stream())
        .map(|(_, _)| {
            let msg = zmq::Message::from_slice(b"").unwrap();
            let mut multipart = VecDeque::new();
            multipart.push_back(msg);
            multipart
        })
        .forward(syncservice.sink::<Error>())
        .and_then(|_| {
            println!("Broadcasting message");

            iter_ok(0..1_000)
                .map(|_| {
                    let msg = zmq::Message::from_slice(b"Rhubarb").unwrap();

                    let mut multipart = VecDeque::new();
                    multipart.push_back(msg);
                    multipart
                })
                .forward(publisher.sink::<Error>())
        })
        .and_then(|_| {
            let msg = zmq::Message::from_slice(b"END").unwrap();

            let mut multipart = VecDeque::new();
            multipart.push_back(msg);

            publisher.send(multipart)
        });

    core.run(runner).unwrap();
}

fn subscriber_thread(thread_num: usize) {
    let mut core = Core::new().unwrap();
    let ctx = Rc::new(zmq::Context::new());

    let port = 5563 + thread_num;

    let pcontroller: Pub = Socket::new(Rc::clone(&ctx), core.handle())
        .bind(&format!("tcp://*:{}", port))
        .try_into()
        .unwrap();

    let scontroller: Sub = Socket::new(Rc::clone(&ctx), core.handle())
        .connect(&format!("tcp://localhost:{}", port))
        .filter(b"")
        .try_into()
        .unwrap();

    let subscriber: Sub = Socket::new(Rc::clone(&ctx), core.handle())
        .connect("tcp://localhost:5561")
        .filter(b"")
        .try_into()
        .unwrap();

    let subscriber = subscriber.controlled(scontroller);

    let syncclient: Req = Socket::new(ctx, core.handle())
        .connect("tcp://localhost:5562")
        .try_into()
        .unwrap();

    let msg = zmq::Message::from_slice(b"").unwrap();
    let mut multipart = VecDeque::new();
    multipart.push_back(msg);

    let recv = syncclient.recv();

    let handle = core.handle();

    let runner =
        syncclient
            .send(multipart)
            .and_then(move |_| recv)
            .and_then(|_| {
                subscriber
                    .stream(Stop)
                    .fold(0, |counter, mut item| {
                        if let Some(msg) = item.pop_front() {
                            if let Some(msg) = msg.as_str() {
                                if msg == "END" {
                                    let msg = zmq::Message::from_slice(b"").unwrap();
                                    let mut multipart = VecDeque::new();
                                    multipart.push_back(msg);
                                    handle.spawn(
                                        pcontroller.send(multipart).map(|_| ()).map_err(|_| ()),
                                    );
                                }
                            }
                        }

                        Ok(counter + 1) as Result<usize, Error>
                    })
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

    for i in 0..SUBSCRIBERS {
        thread::sleep(Duration::from_secs(1));
        threads.push(thread::spawn(move || subscriber_thread(i)));
    }

    for thread in threads {
        thread.join().unwrap();
    }
}
