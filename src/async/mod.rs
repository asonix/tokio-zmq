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

use zmq;
use futures::{Future, Stream};

mod stream;
mod sink;
mod future;

pub use self::stream::ZmqStream;
pub use self::sink::ZmqSink;
pub use self::future::ZmqResponse;

pub trait RepHandler: Clone {
    type Request: From<zmq::Message>;
    type Response: Into<zmq::Message>;
    type Error: From<()> + Sized + Debug;

    type Future: Future<Item = Self::Response, Error = Self::Error>;

    fn call(&self, req: Self::Request) -> Self::Future;
}

pub struct RepBuilder {
    sock: Rc<zmq::Socket>,
}

impl RepBuilder {
    pub fn new(sock: Rc<zmq::Socket>) -> Self {
        RepBuilder { sock }
    }

    pub fn handler<H>(self, handler: H) -> RepServerBuilder<H>
    where
        H: RepHandler,
    {
        RepServerBuilder {
            sock: self.sock,
            handler: handler,
        }
    }

    pub fn connect(self, addr: &str) -> zmq::Result<RepClient> {
        self.sock.connect(addr)?;

        Ok(RepClient { sock: self.sock })
    }
}

pub struct RepServerBuilder<H>
where
    H: RepHandler,
{
    sock: Rc<zmq::Socket>,
    handler: H,
}

impl<H> RepServerBuilder<H>
where
    H: RepHandler,
{
    pub fn bind(self, addr: &str) -> zmq::Result<RepServer<H>> {
        self.sock.bind(addr)?;

        Ok(RepServer {
            sock: self.sock,
            handler: self.handler,
        })
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
    pub fn new(sock: Rc<zmq::Socket>) -> RepBuilder {
        RepBuilder::new(sock)
    }

    pub fn runner(
        &self,
    ) -> impl Future<
        Item = (impl Stream<Item = zmq::Message, Error = H::Error>, ZmqSink<H>),
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

    pub fn sink(&self) -> ZmqSink<H> {
        ZmqSink::new(Rc::clone(&self.sock))
    }
}

pub struct RepClient {
    sock: Rc<zmq::Socket>,
}

impl RepClient {
    pub fn new(sock: Rc<zmq::Socket>) -> RepBuilder {
        RepBuilder::new(sock)
    }

    pub fn send(&self, msg: zmq::Message) -> ZmqResponse {
        ZmqResponse::new(Rc::clone(&self.sock), msg)
    }
}
