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

use std::rc::Rc;

use async::sink::ZmqSink;
use async::future::ZmqRequest;

use zmq;
use futures::Future;

pub struct Pub {
    sock: Rc<zmq::Socket>,
}

impl Pub {
    pub fn new() -> PubBuilder {
        PubBuilder::new()
    }

    pub fn send(&self, msg: zmq::Message) -> impl Future<Item = (), Error = ()> {
        ZmqRequest::new(Rc::clone(&self.sock), msg)
    }

    pub fn sink<H>(&self) -> ZmqSink<H> {
        ZmqSink::new(Rc::clone(&self.sock))
    }
}

pub enum PubBuilder {
    Sock(Rc<zmq::Socket>),
    Fail(zmq::Error),
}

impl PubBuilder {
    pub fn new() -> Self {
        let context = zmq::Context::new();
        match context.socket(zmq::PUB) {
            Ok(sock) => PubBuilder::Sock(Rc::new(sock)),
            Err(e) => PubBuilder::Fail(e),
        }
    }

    pub fn bind(self, addr: &str) -> zmq::Result<Pub> {
        match self {
            PubBuilder::Sock(sock) => {
                sock.bind(addr)?;

                Ok(Pub { sock })
            }
            PubBuilder::Fail(e) => Err(e),
        }
    }
}
