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
use std::env;

use futures::stream::iter_ok;
use futures::{Future, Stream};
use tokio_core::reactor::Core;
use tokio_zmq::prelude::*;
use tokio_zmq::{Dealer, Rep, Req, Router, Pub, Sub};
use tokio_zmq::{Socket, Multipart, Error};

pub struct Stop;

impl ControlHandler for Stop {
    fn should_stop(&mut self, _: Multipart) -> bool {
        println!("Got stop signal");
        true
    }
}

fn client() {
    let mut core = Core::new().unwrap();
    let ctx = Rc::new(zmq::Context::new());
    let handle = core.handle();
    let req: Req = Socket::create(Rc::clone(&ctx), &handle)
        .connect("tcp://localhost:5559")
        .try_into()
        .unwrap();

    let zpub: Pub = Socket::create(Rc::clone(&ctx), &handle)
        .bind("tcp://*:5561")
        .try_into()
        .unwrap();

    let runner = iter_ok(0..10)
        .and_then(|request_nbr| {
            let msg = zmq::Message::from_slice(b"Hewwo?").unwrap();

            println!("Sending 'Hewwo?' for {}", request_nbr);

            let response = req.recv();
            let request = req.send(msg.into());

            request.and_then(move |_| {
                response.map(move |multipart| (request_nbr, multipart))
            })
        })
        .for_each(|(request_nbr, multipart)| {
            for msg in multipart {
                if let Some(msg) = msg.as_str() {
                    println!("Received reply {} {}", request_nbr, msg);
                }
            }

            Ok(())
        })
        .and_then(|_| {
            let msg = zmq::Message::from_slice(b"").unwrap();

            zpub.send(msg.into())
        });

    let res = core.run(runner);
    if let Err(e) = res {
        println!("client bailed: {:?}", e);
    }
}

fn worker() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let ctx = Rc::new(zmq::Context::new());

    let rep: Rep = Socket::create(Rc::clone(&ctx), &handle)
        .connect("tcp://localhost:5560")
        .try_into()
        .unwrap();

    let sub: Sub = Socket::create(Rc::clone(&ctx), &handle)
        .connect("tcp://localhost:5561")
        .filter(b"")
        .try_into()
        .unwrap();

    let rep = rep.controlled(sub);

    let runner = rep.stream(Stop)
        .map(|multipart| {
            for msg in multipart {
                if let Some(msg) = msg.as_str() {
                    println!("Received request: {}", msg);
                }
            }

            let msg = zmq::Message::from_slice(b"Mr Obama???").unwrap();

            msg.into()
        })
        .forward(rep.sink::<Error>());

    let res = core.run(runner);

    if let Err(e) = res {
        println!("worker bailed: {:?}", e);
    }
}

fn broker() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let ctx = Rc::new(zmq::Context::new());

    let router: Router = Socket::create(Rc::clone(&ctx), &handle)
        .bind("tcp://*:5559")
        .try_into()
        .unwrap();
    let sub: Sub = Socket::create(Rc::clone(&ctx), &handle)
        .connect("tcp://localhost:5561")
        .filter(b"")
        .try_into()
        .unwrap();
    let router = router.controlled(sub);

    let dealer: Dealer = Socket::create(Rc::clone(&ctx), &handle)
        .bind("tcp://*:5560")
        .try_into()
        .unwrap();
    let sub: Sub = Socket::create(Rc::clone(&ctx), &handle)
        .connect("tcp://localhost:5561")
        .filter(b"")
        .try_into()
        .unwrap();
    let dealer = dealer.controlled(sub);

    let d2r = dealer
        .stream(Stop)
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
        .forward(router.sink::<Error>());
    let r2d = router
        .stream(Stop)
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
        .forward(dealer.sink::<Error>());

    handle.spawn(d2r.map(|_| ()).map_err(|e| println!("d2r bailed: {:?}", e)));
    let res = core.run(r2d);

    if let Err(e) = res {
        println!("broker bailed: {:?}", e);
    }
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
