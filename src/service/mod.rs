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

use std::fmt::Debug;

use zmq;
use futures::{Future, Stream};

use async::{ControlHandler, MsgStream, ZmqSink};
use super::{Controlled, StreamSocket, SinkSocket};

pub mod response;

pub trait Handler: Clone {
    type Request: From<MsgStream<Self::Error>>;
    type Response: Stream<Item = zmq::Message, Error = Self::Error>;
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
    P: Controlled + StreamSocket + 'a,
    C: SinkSocket + 'a,
    H: Handler,
{
    pub fn run_controlled<S>(
        &self,
        controll_handler: S,
    ) -> impl Future<
        Item = (impl Stream<
            Item = impl Stream<Item = zmq::Message, Error = H::Error>,
            Error = H::Error,
        >,
                ZmqSink<H::Response, H::Error>),
        Error = H::Error,
    >
    where
        S: ControlHandler,
    {
        let handler = self.handler.clone();

        self.stream
            .controlled_stream::<S, H::Error>(controll_handler)
            .and_then(move |msg| handler.call(msg.into()))
            .map(|msg| msg.into())
            .map_err(|e| e.into())
            .forward(self.sink.sink::<H::Response, H::Error>())
    }
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
        Item = (impl Stream<
            Item = impl Stream<Item = zmq::Message, Error = H::Error>,
            Error = H::Error,
        >,
                ZmqSink<H::Response, H::Error>),
        Error = H::Error,
    > {
        let handler = self.handler.clone();

        self.stream
            .stream::<H::Error>()
            .and_then(move |msg| handler.call(msg.into()))
            .map(|msg| msg.into())
            .map_err(|e| e.into())
            .forward(self.sink.sink::<H::Response, H::Error>())
    }
}
