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

use async::stream::ZmqStream;

use zmq;

pub struct Pull {
    sock: Rc<zmq::Socket>,
}

impl Pull {
    pub fn new() -> PullBuilder {
        PullBuilder::new()
    }

    pub fn stream(&self) -> ZmqStream {
        ZmqStream::new(Rc::clone(&self.sock))
    }
}

pub enum PullBuilder {
    Sock(Rc<zmq::Socket>),
    Fail(zmq::Error),
}

impl PullBuilder {
    pub fn new() -> Self {
        let context = zmq::Context::new();
        match context.socket(zmq::PULL) {
            Ok(sock) => PullBuilder::Sock(Rc::new(sock)),
            Err(e) => PullBuilder::Fail(e),
        }
    }

    pub fn bind(self, addr: &str) -> zmq::Result<Pull> {
        match self {
            PullBuilder::Sock(sock) => {
                sock.bind(&addr)?;

                Ok(Pull { sock })
            }
            PullBuilder::Fail(e) => Err(e),
        }
    }

    pub fn connect(self, addr: &str) -> zmq::Result<Pull> {
        match self {
            PullBuilder::Sock(sock) => {
                sock.connect(&addr)?;

                Ok(Pull { sock })
            }
            PullBuilder::Fail(e) => Err(e),
        }
    }
}
