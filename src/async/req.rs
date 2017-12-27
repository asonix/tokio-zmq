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

use async::future::ZmqResponse;

use zmq;

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

    pub fn connect(self, addr: &str) -> zmq::Result<ReqClient> {
        match self {
            ReqBuilder::Sock(sock) => {
                sock.connect(addr)?;

                Ok(ReqClient { sock: sock })
            }
            ReqBuilder::Fail(e) => Err(e),
        }
    }
}

pub struct ReqClient {
    sock: Rc<zmq::Socket>,
}

impl ReqClient {
    pub fn new() -> ReqBuilder {
        ReqBuilder::new()
    }

    pub fn send(&self, msg: zmq::Message) -> ZmqResponse {
        ZmqResponse::new(Rc::clone(&self.sock), msg)
    }
}
