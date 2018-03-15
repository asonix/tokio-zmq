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
extern crate futures;
extern crate log;
extern crate tokio;
extern crate tokio_zmq;
extern crate zmq;

use std::convert::TryInto;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use futures::stream::iter_ok;
use futures::{FutureExt, SinkExt, StreamExt};
use tokio_zmq::prelude::*;
use tokio_zmq::{Pub, Rep, Req, Sub};
use tokio_zmq::{Error, Multipart, Socket};

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
    let ctx = Arc::new(zmq::Context::new());

    let publisher: Pub = Socket::builder(Arc::clone(&ctx))
        .bind("tcp://*:5561")
        .try_into()
        .unwrap();

    let syncservice: Rep = Socket::builder(ctx)
        .bind("tcp://*:5562")
        .try_into()
        .unwrap();

    println!("Waiting for subscribers");

    let (sync_sink, sync_stream) = syncservice.sink_stream().split();

    let runner = iter_ok(0..SUBSCRIBERS)
        .zip(sync_stream)
        .map(|(_, _)| zmq::Message::from_slice(b"").unwrap().into())
        .forward(sync_sink)
        .and_then(move |_| {
            println!("Broadcasting message");

            iter_ok(0..MESSAGES)
                .map(|_| zmq::Message::from_slice(b"Rhubarb").unwrap().into())
                .forward(publisher.sink())
        })
        .and_then(|(_stream, sink)| {
            let msg = zmq::Message::from_slice(b"END").unwrap();

            sink.send(msg.into())
        });

    tokio::runtime::run2(runner.map(|_| ()).or_else(|e| {
        println!("Error in publisher: {:?}", e);
        Ok(())
    }));
}

fn subscriber_thread() {
    let ctx = Arc::new(zmq::Context::new());

    let subscriber: Sub = Socket::builder(Arc::clone(&ctx))
        .connect("tcp://localhost:5561")
        .filter(b"")
        .try_into()
        .unwrap();

    let syncclient: Req = Socket::builder(ctx)
        .connect("tcp://localhost:5562")
        .try_into()
        .unwrap();

    let msg = zmq::Message::from_slice(b"").unwrap();

    let runner = syncclient
        .send(msg.into())
        .and_then(|(sock, file)| Socket::from_sock_and_file(sock, file).recv())
        .and_then(move |_| {
            subscriber
                .stream()
                .with_end(Stop)
                .fold(0, |counter, _| Ok(counter + 1) as Result<usize, Error>)
                .and_then(|total| {
                    println!("Received {} updates", total);
                    Ok(())
                })
        });

    tokio::runtime::run2(runner.map(|_| ()).or_else(|e| {
        println!("Error in subscriber: {:?}", e);
        Ok(())
    }));
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
