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
extern crate tokio_core;
extern crate tokio_zmq;
extern crate zmq;

use std::env;
use std::convert::{TryFrom, TryInto};
use std::rc::Rc;
use std::thread;
use std::time::Duration;

use futures::{Future, Sink, Stream};
use futures::sync::mpsc;
use tokio_core::reactor::Core;
use tokio_zmq::prelude::*;
use tokio_zmq::{Pub, Req, Router, Sub};
use tokio_zmq::{Multipart, Socket};

const NUM_CLIENTS: usize = 1000;
const NUM_WORKERS: usize = 3;

/* ----------------------------------Error----------------------------------- */

#[derive(Debug)]
enum Error {
    Zmq(zmq::Error),
    TokioZmq(tokio_zmq::Error),
    WorkerSend,
    WorkerRecv,
    NotEnoughMessages,
    TooManyMessages,
    MsgNotEmpty,
}

impl From<tokio_zmq::Error> for Error {
    fn from(e: tokio_zmq::Error) -> Self {
        Error::TokioZmq(e)
    }
}

impl From<zmq::Error> for Error {
    fn from(e: zmq::Error) -> Self {
        Error::Zmq(e)
    }
}

/* --------------------------------Envelope---------------------------------- */

struct Envelope {
    addr: zmq::Message,
    empty: zmq::Message,
    request: zmq::Message,
}

impl Envelope {
    fn addr(&self) -> &zmq::Message {
        &self.addr
    }

    fn request(&self) -> &zmq::Message {
        &self.request
    }

    fn set_request(&mut self, msg: zmq::Message) {
        self.request = msg;
    }
}

impl TryFrom<Multipart> for Envelope {
    type Error = Error;

    fn try_from(mut m: Multipart) -> Result<Self, Self::Error> {
        let addr = m.pop_front().ok_or(Error::NotEnoughMessages)?;
        let empty = m.pop_front().ok_or(Error::NotEnoughMessages)?;
        if !empty.is_empty() {
            return Err(Error::MsgNotEmpty);
        }
        let request = m.pop_front().ok_or(Error::NotEnoughMessages)?;

        if !m.is_empty() {
            return Err(Error::TooManyMessages);
        }

        Ok(Envelope {
            addr,
            empty,
            request,
        })
    }
}

impl From<Envelope> for Multipart {
    fn from(e: Envelope) -> Self {
        let mut multipart = Multipart::new();

        multipart.push_back(e.addr);
        multipart.push_back(e.empty);
        multipart.push_back(e.request);

        multipart
    }
}

/* -----------------------------------Stop----------------------------------- */

struct Stop;

impl ControlHandler for Stop {
    fn should_stop(&mut self, _: Multipart) -> bool {
        println!("Received stop signal!");
        true
    }
}

/* ----------------------------------client---------------------------------- */

fn client_task(client_num: usize) -> usize {
    let mut core = Core::new().unwrap();
    let context = Rc::new(zmq::Context::new());

    let client: Req = Socket::builder(context, &core.handle())
        .identity(format!("c{}", client_num).as_bytes())
        .connect("tcp://localhost:5672")
        .try_into()
        .unwrap();

    let msg = zmq::Message::from_slice(b"HELLO").unwrap();
    let resp = client.recv();
    let fut = client
        .send(msg.into())
        .and_then(|_| resp)
        .and_then(|multipart| {
            if let Some(msg) = multipart.get(0) {
                println!("Client {}: {:?}", client_num, msg.as_str());
            }
            Ok(())
        });

    core.run(fut).unwrap();
    client_num
}

/* ----------------------------------worker---------------------------------- */

fn worker_task(worker_num: usize) -> usize {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let context = Rc::new(zmq::Context::new());

    let control: Sub = Socket::builder(Rc::clone(&context), &handle)
        .connect("tcp://localhost:5674")
        .filter(b"")
        .try_into()
        .unwrap();
    let worker: Req = Socket::builder(context, &handle)
        .identity(format!("w{}", worker_num).as_bytes())
        .connect("tcp://localhost:5673")
        .try_into()
        .unwrap();

    let msg = zmq::Message::from_slice(b"READY").unwrap();
    let fut = worker.send(msg.into()).map_err(Error::from);

    let inner_fut = worker
        .stream()
        .controlled(control, Stop)
        .map_err(Error::from)
        .and_then(|multipart| {
            let mut envelope: Envelope = multipart.try_into()?;

            println!(
                "Worker: {:?} from {:?}",
                envelope.request().as_str(),
                envelope.addr().as_str()
            );

            let msg = zmq::Message::from_slice(b"OK")?;
            envelope.set_request(msg);

            Ok(envelope.into())
        })
        .forward(worker.sink::<Error>());

    core.run(fut.and_then(|_| inner_fut)).unwrap();
    println!("Worker {} is done", worker_num);
    worker_num
}

/* ----------------------------------broker---------------------------------- */

fn broker_task() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let context = Rc::new(zmq::Context::new());

    let frontend: Router = Socket::builder(Rc::clone(&context), &handle)
        .bind("tcp://*:5672")
        .try_into()
        .unwrap();

    let control: Sub = Socket::builder(Rc::clone(&context), &handle)
        .connect("tcp://localhost:5674")
        .filter(b"")
        .try_into()
        .unwrap();

    let backend: Router = Socket::builder(context, &handle)
        .bind("tcp://*:5673")
        .try_into()
        .unwrap();

    let (worker_send, worker_recv) = mpsc::channel::<zmq::Message>(10);

    let back2front = backend
        .stream()
        .controlled(control, Stop)
        .map_err(Error::from)
        .and_then(|mut multipart| {
            let worker_id = multipart.pop_front().ok_or(Error::NotEnoughMessages)?;

            Ok((multipart, worker_id))
        })
        .and_then(|(multipart, worker_id)| {
            worker_send
                .clone()
                .send(worker_id)
                .map(|_| multipart)
                .map_err(|_| Error::WorkerSend)
        })
        .and_then(|mut multipart| {
            let empty = multipart.pop_front().ok_or(Error::NotEnoughMessages)?;
            assert!(empty.is_empty());
            let client_id = multipart.pop_front().ok_or(Error::NotEnoughMessages)?;

            Ok((multipart, client_id))
        })
        .filter_map(|(multipart, client_id)| {
            if &*client_id == b"READY" {
                None
            } else {
                Some((multipart, client_id))
            }
        })
        .and_then(|(mut multipart, client_id)| {
            let empty = multipart.pop_front().ok_or(Error::NotEnoughMessages)?;
            assert!(empty.is_empty());
            let reply = multipart.pop_front().ok_or(Error::NotEnoughMessages)?;

            let mut response = Multipart::new();

            response.push_back(client_id);
            response.push_back(empty);
            response.push_back(reply);

            Ok(response)
        })
        .forward(frontend.sink::<Error>());

    let front2back = frontend
        .stream()
        .map_err(Error::from)
        .zip(worker_recv.map_err(|_| Error::WorkerRecv))
        .and_then(|(mut multipart, worker_id)| {
            let client_id = multipart.pop_front().ok_or(Error::NotEnoughMessages)?;
            let empty = multipart.pop_front().ok_or(Error::NotEnoughMessages)?;
            assert!(empty.is_empty());
            let request = multipart.pop_front().ok_or(Error::NotEnoughMessages)?;

            let mut response = Multipart::new();

            response.push_back(worker_id);
            response.push_back(empty);
            response.push_back(client_id);
            response.push_back(zmq::Message::new()?);
            response.push_back(request);

            Ok(response)
        })
        .forward(backend.sink::<Error>())
        .map(|_| ())
        .map_err(|e| println!("Error: {:?}", e));

    handle.spawn(front2back);

    core.run(back2front).unwrap();
    println!("Broker is done");
}

/* --------------------------------use_broker-------------------------------- */

#[derive(Clone, Debug, Eq, PartialEq)]
enum UseBroker {
    Yes,
    No,
    All,
}

/* -----------------------------------main----------------------------------- */

fn main() {
    env_logger::init().unwrap();

    let use_broker = match env::var("USE_BROKER")
        .unwrap_or_else(|_| "all".to_owned())
        .as_str()
    {
        "yes" => UseBroker::Yes,
        "no" => UseBroker::No,
        _ => UseBroker::All,
    };

    let mut broker_thread = None;

    match use_broker {
        UseBroker::Yes | UseBroker::All => {
            // Spawn threads
            broker_thread = Some(thread::spawn(broker_task));
        }
        _ => (),
    }

    match use_broker {
        UseBroker::No | UseBroker::All => {
            // Set up control socket
            let mut core = Core::new().unwrap();
            let context = Rc::new(zmq::Context::new());
            let control: Pub = Socket::builder(context, &core.handle())
                .bind("tcp://*:5674")
                .try_into()
                .unwrap();

            let workers = (0..NUM_WORKERS)
                .map(|worker_num| thread::spawn(move || worker_task(worker_num)))
                .collect::<Vec<_>>();

            let clients = (0..NUM_CLIENTS)
                .map(|client_num| {
                    if client_num % 20 == 0 {
                        println!("Sleeping to avoid too many open files");
                        thread::sleep(Duration::from_millis(20));
                    }
                    thread::spawn(move || client_task(client_num))
                })
                .collect::<Vec<_>>();

            // Wait for clients to finish
            for client in clients {
                client.join().unwrap();
            }

            // Signal end when all clients have joined
            core.run(control.send(zmq::Message::new().unwrap().into()))
                .unwrap();

            for worker in workers {
                let worker_num = worker.join().unwrap();
                println!("Joined Worker {}", worker_num);
            }
        }
        _ => (),
    }

    if let Some(broker_thread) = broker_thread {
        broker_thread.join().unwrap();
        println!("Joined Broker");
    }
}
