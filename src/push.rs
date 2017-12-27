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

pub struct Push {
    sock: Rc<zmq::Socket>,
}

impl Push {
    pub fn new() -> PushBuilder {
        PushBuilder::new()
    }

    pub fn send(&self, msg: zmq::Message) -> impl Future<Item = (), Error = zmq::Error> {
        ZmqRequest::new(Rc::clone(&self.sock), msg)
    }

    pub fn sink<E>(&self) -> ZmqSink<E>
    where
        E: From<zmq::Error>,
    {
        ZmqSink::new(Rc::clone(&self.sock))
    }
}

pub enum PushBuilder {
    Sock(Rc<zmq::Socket>),
    Fail(zmq::Error),
}

impl PushBuilder {
    pub fn new() -> Self {
        let context = zmq::Context::new();
        match context.socket(zmq::PUSH) {
            Ok(sock) => PushBuilder::Sock(Rc::new(sock)),
            Err(e) => PushBuilder::Fail(e),
        }
    }

    pub fn bind(self, addr: &str) -> zmq::Result<Push> {
        match self {
            PushBuilder::Sock(sock) => {
                sock.bind(&addr)?;

                Ok(Push { sock })
            }
            PushBuilder::Fail(e) => Err(e),
        }
    }

    pub fn connect(self, addr: &str) -> zmq::Result<Push> {
        match self {
            PushBuilder::Sock(sock) => {
                sock.connect(addr)?;

                Ok(Push { sock })
            }
            PushBuilder::Fail(e) => Err(e),
        }
    }
}
