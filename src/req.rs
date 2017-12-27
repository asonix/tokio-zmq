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

use async::future::{ZmqRequest, ZmqResponse};

use zmq;
use futures::Future;

pub enum ReqBuilder {
    Sock(Rc<zmq::Socket>),
    Fail(zmq::Error),
}

impl ReqBuilder {
    pub fn new() -> Self {
        let context = zmq::Context::new();
        match context.socket(zmq::REQ) {
            Ok(sock) => ReqBuilder::Sock(Rc::new(sock)),
            Err(e) => ReqBuilder::Fail(e),
        }
    }

    pub fn connect(self, addr: &str) -> zmq::Result<Req> {
        match self {
            ReqBuilder::Sock(sock) => {
                sock.connect(addr)?;

                Ok(Req { sock: sock })
            }
            ReqBuilder::Fail(e) => Err(e),
        }
    }
}

pub struct Req {
    sock: Rc<zmq::Socket>,
}

impl Req {
    pub fn new() -> ReqBuilder {
        ReqBuilder::new()
    }

    pub fn send(&self, msg: zmq::Message) -> impl Future<Item = zmq::Message, Error = zmq::Error> {
        let sock = Rc::clone(&self.sock);
        ZmqRequest::new(Rc::clone(&self.sock), msg).and_then(|_| ZmqResponse::new(sock))
    }
}
