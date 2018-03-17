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
extern crate tokio_executor;
extern crate tokio_zmq;
extern crate zmq;

use std::convert::TryInto;
use std::env;
use std::sync::Arc;
use std::thread;

use futures::stream::iter_ok;
use futures::{FutureExt, StreamExt};
use tokio_zmq::prelude::*;
use tokio_zmq::{Dealer, Pub, Rep, Req, Router, Sub};
use tokio_zmq::{Multipart, Socket};

const CLIENT_REQUESTS: usize = 1000;

pub struct Stop;

impl ControlHandler for Stop {
    fn should_stop(&mut self, _: Multipart) -> bool {
        println!("Got stop signal");
        true
    }
}

fn client() {
    let ctx = Arc::new(zmq::Context::new());
    let req: Req = Socket::builder(Arc::clone(&ctx))
        .connect("tcp://localhost:5559")
        .try_into()
        .unwrap();

    let zpub: Pub = Socket::builder(Arc::clone(&ctx))
        .bind("tcp://*:5561")
        .try_into()
        .unwrap();

    println!("Sending 'Hewwo?' for 0");
    let runner = req.send(zmq::Message::from_slice(b"Hewwo?").unwrap().into())
        .and_then(|(sock, file)| {
            let (sink, stream) = Socket::from_sock_and_file(sock, file).sink_stream().split();

            stream
                .zip(iter_ok(1..CLIENT_REQUESTS))
                .map(|(multipart, request_nbr)| {
                    for msg in multipart {
                        if let Some(msg) = msg.as_str() {
                            println!("Received reply {} {}", request_nbr, msg);
                        }
                    }

                    println!("Sending 'Hewwo?' for {}", request_nbr);
                    zmq::Message::from_slice(b"Hewwo?").unwrap().into()
                })
                .forward(sink)
        })
        .and_then(move |(stream, _sink)| {
            stream
                .into_future()
                .map_err(|(e, _)| e)
                .and_then(|(maybe_last_item, _stream)| {
                    if let Some(multipart) = maybe_last_item {
                        for msg in multipart {
                            if let Some(msg) = msg.as_str() {
                                println!("Received last reply {}", msg);
                            }
                        }
                    }

                    let msg = zmq::Message::from_slice(b"").unwrap();

                    zpub.send(msg.into())
                })
        });

    tokio::runtime::run2(runner.map(|_| ()).or_else(|e| {
        println!("Error in client: {:?}", e);
        Ok(())
    }));
}

fn worker() {
    let ctx = Arc::new(zmq::Context::new());

    let rep: Rep = Socket::builder(Arc::clone(&ctx))
        .connect("tcp://localhost:5560")
        .try_into()
        .unwrap();

    let cmd: Sub = Socket::builder(Arc::clone(&ctx))
        .connect("tcp://localhost:5561")
        .filter(b"")
        .try_into()
        .unwrap();

    let (rep_sink, rep_stream) = rep.sink_stream().split();

    let runner = rep_stream
        .controlled(cmd.stream(), Stop)
        .map(|multipart| {
            for msg in multipart {
                if let Some(msg) = msg.as_str() {
                    println!("Received request: {}", msg);
                }
            }

            let msg = zmq::Message::from_slice(b"Mr Obama???").unwrap();

            msg.into()
        })
        .forward(rep_sink);

    tokio::runtime::run2(runner.map(|_| ()).or_else(|e| {
        println!("Error in worker: {:?}", e);
        Ok(())
    }));
}

fn broker() {
    let ctx = Arc::new(zmq::Context::new());

    let router: Router = Socket::builder(Arc::clone(&ctx))
        .bind("tcp://*:5559")
        .try_into()
        .unwrap();

    let dealer: Dealer = Socket::builder(Arc::clone(&ctx))
        .bind("tcp://*:5560")
        .try_into()
        .unwrap();

    let cmd1: Sub = Socket::builder(Arc::clone(&ctx))
        .connect("tcp://localhost:5561")
        .filter(b"")
        .try_into()
        .unwrap();
    let cmd2: Sub = Socket::builder(Arc::clone(&ctx))
        .connect("tcp://localhost:5561")
        .filter(b"")
        .try_into()
        .unwrap();

    let (dealer_sink, dealer_stream) = dealer.sink_stream().split();
    let (router_sink, router_stream) = router.sink_stream().split();

    let d2r = dealer_stream
        .controlled(cmd1.stream(), Stop)
        .map(|multipart| {
            for msg in &multipart {
                if let Some(msg) = msg.as_str() {
                    println!("Relaying message '{}' to router", msg);
                } else {
                    println!("Relaying unknown message to router");
                }
            }
            multipart
        })
        .forward(router_sink);

    let r2d = router_stream
        .controlled(cmd2.stream(), Stop)
        .map(|multipart| {
            for msg in &multipart {
                if let Some(msg) = msg.as_str() {
                    println!("Relaying message '{}' to dealer", msg);
                } else {
                    println!("Relaying unknown message to dealer");
                }
            }
            multipart
        })
        .forward(dealer_sink);

    tokio::runtime::run2(d2r.join(r2d).map(|_| ()).or_else(|e| {
        println!("broker bailed: {:?}", e);
        Ok(())
    }));
}

#[derive(Debug, PartialEq)]
enum Selection {
    All,
    Broker,
    Worker,
    Client,
}

impl Selection {
    fn broker(&self) -> bool {
        *self == Selection::All || *self == Selection::Broker
    }

    fn worker(&self) -> bool {
        *self == Selection::All || *self == Selection::Worker
    }

    fn client(&self) -> bool {
        *self == Selection::All || *self == Selection::Client
    }
}

fn main() {
    env_logger::init().unwrap();

    let selection = env::var("SELECTION").unwrap_or_else(|_| "all".into());

    let selection = match selection.as_ref() {
        "broker" => Selection::Broker,
        "worker" => Selection::Worker,
        "client" => Selection::Client,
        _ => Selection::All,
    };

    println!("SELECTION: {:?}", selection);

    let mut broker_thread = None;
    let mut worker_thread = None;
    let mut client_thread = None;

    if selection.broker() {
        broker_thread = Some(thread::spawn(broker));
    }
    if selection.worker() {
        worker_thread = Some(thread::spawn(worker));
    }
    if selection.client() {
        client_thread = Some(thread::spawn(client));
    }

    if let Some(broker_thread) = broker_thread {
        broker_thread.join().unwrap();
    }
    if let Some(worker_thread) = worker_thread {
        worker_thread.join().unwrap();
    }
    if let Some(client_thread) = client_thread {
        client_thread.join().unwrap();
    }
}
