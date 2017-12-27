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
use std::fmt::Debug;

use async::stream::ZmqStream;
use async::sink::ZmqSink;

use zmq;
use futures::{Future, Stream};

pub trait RepHandler: Clone {
    type Request: From<zmq::Message>;
    type Response: Into<zmq::Message>;
    type Error: From<()> + Sized + Debug;

    type Future: Future<Item = Self::Response, Error = Self::Error>;

    fn call(&self, req: Self::Request) -> Self::Future;
}

pub enum RepBuilder {
    Sock(Rc<zmq::Socket>),
    Fail(zmq::Error),
}

impl RepBuilder {
    pub fn new() -> Self {
        let context = zmq::Context::new();

        match context.socket(zmq::REP) {
            Ok(sock) => RepBuilder::Sock(Rc::new(sock)),
            Err(e) => RepBuilder::Fail(e),
        }
    }

    pub fn handler<H>(self, handler: H) -> RepServerBuilder<H>
    where
        H: RepHandler,
    {
        match self {
            RepBuilder::Sock(sock) => RepServerBuilder::Server { sock, handler },
            RepBuilder::Fail(e) => RepServerBuilder::Fail(e),
        }
    }
}

pub enum RepServerBuilder<H>
where
    H: RepHandler,
{
    Server { sock: Rc<zmq::Socket>, handler: H },
    Fail(zmq::Error),
}

impl<H> RepServerBuilder<H>
where
    H: RepHandler,
{
    pub fn bind(self, addr: &str) -> zmq::Result<RepServer<H>> {
        match self {
            RepServerBuilder::Server { sock, handler } => {
                sock.bind(addr)?;

                Ok(RepServer { sock, handler })
            }
            RepServerBuilder::Fail(e) => Err(e),
        }
    }
}

pub struct RepServer<H>
where
    H: RepHandler,
{
    sock: Rc<zmq::Socket>,
    handler: H,
}

impl<H> RepServer<H>
where
    H: RepHandler,
{
    pub fn new() -> RepBuilder {
        RepBuilder::new()
    }

    pub fn runner(
        &self,
    ) -> impl Future<
        Item = (impl Stream<Item = zmq::Message, Error = H::Error>, ZmqSink<H::Error>),
        Error = H::Error,
    > {
        let handler = self.handler.clone();

        self.stream()
            .map_err(H::Error::from)
            .and_then(move |msg| handler.call(msg.into()))
            .map(|msg| msg.into())
            .map_err(|e| e.into())
            .forward(self.sink())
    }

    pub fn stream(&self) -> ZmqStream {
        ZmqStream::new(Rc::clone(&self.sock))
    }

    pub fn sink(&self) -> ZmqSink<H::Error> {
        ZmqSink::new(Rc::clone(&self.sock))
    }
}
