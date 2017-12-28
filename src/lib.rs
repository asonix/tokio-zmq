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

#![feature(conservative_impl_trait)]

extern crate zmq;
#[macro_use]
extern crate zmq_futures_derive;
extern crate futures;

pub mod async;
pub mod rep;
pub mod req;
pub mod zpub;
pub mod sub;
pub mod push;
pub mod pull;

use std::rc::Rc;
use std::fmt::Debug;

use futures::{Future, Stream};

use async::{MsgStream, ZmqRequest, ZmqResponse, ZmqSink, ZmqStream};

pub trait Handler: Clone {
    type Request: From<MsgStream>;
    type Response: Into<zmq::Message>;
    type Error: From<zmq::Error> + Sized + Debug;

    type Future: Future<Item = Self::Response, Error = Self::Error>;

    fn call(&self, req: Self::Request) -> Self::Future;
}

pub struct Runner<'a, P, C, H>
where
    P: StreamSocket + 'a,
    C: SinkSocket + 'a,
    H: Handler,
{
    stream: &'a P,
    sink: &'a C,
    handler: H,
}

impl<'a, P, C, H> Runner<'a, P, C, H>
where
    P: StreamSocket + 'a,
    C: SinkSocket + 'a,
    H: Handler,
{
    pub fn new(stream: &'a P, sink: &'a C, handler: H) -> Self {
        Runner {
            stream,
            sink,
            handler,
        }
    }

    pub fn run(
        &self,
    ) -> impl Future<
        Item = (impl Stream<Item = zmq::Message, Error = H::Error>, ZmqSink<H::Error>),
        Error = H::Error,
    > {
        let handler = self.handler.clone();

        self.stream
            .stream()
            .map_err(H::Error::from)
            .and_then(move |msg| handler.call(msg.into()))
            .map(|msg| msg.into())
            .map_err(|e| e.into())
            .forward(self.sink.sink::<H::Error>())
    }
}

pub trait ZmqSocket {
    fn socket(&self) -> Rc<zmq::Socket>;
}

pub trait StreamSocket: ZmqSocket {
    fn recv(&self) -> async::ZmqResponse {
        ZmqResponse::new(self.socket())
    }

    fn stream(&self) -> async::ZmqStream {
        ZmqStream::new(self.socket())
    }
}

pub trait SinkSocket: ZmqSocket {
    fn send(&self, msg: zmq::Message) -> ZmqRequest {
        ZmqRequest::new(self.socket(), msg)
    }

    fn sink<E>(&self) -> async::ZmqSink<E>
    where
        E: From<zmq::Error>,
    {
        ZmqSink::new(self.socket())
    }
}
