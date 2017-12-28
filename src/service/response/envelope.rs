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

use std::marker::PhantomData;
use std::collections::VecDeque;

use zmq;
use futures::{Async, AsyncSink, Poll, Sink, StartSend, Stream, task};

pub struct Envelope<E> {
    key: Option<zmq::Message>,
    addr: Option<zmq::Message>,
    contents: VecDeque<zmq::Message>,
    phantom: PhantomData<E>,
}

impl<E> Envelope<E> {
    pub fn new(key: zmq::Message) -> Self {
        Envelope {
            key: Some(key),
            addr: None,
            contents: VecDeque::new(),
            phantom: PhantomData,
        }
    }

    pub fn new_with_addr(key: zmq::Message, addr: zmq::Message) -> Self {
        Envelope {
            key: Some(key),
            addr: Some(addr),
            contents: VecDeque::new(),
            phantom: PhantomData,
        }
    }

    pub fn new_with_contents(key: zmq::Message, contents: VecDeque<zmq::Message>) -> Self {
        Envelope {
            key: Some(key),
            addr: None,
            contents: contents,
            phantom: PhantomData,
        }
    }

    pub fn new_with_addr_and_contents(
        key: zmq::Message,
        addr: zmq::Message,
        contents: VecDeque<zmq::Message>,
    ) -> Self {
        Envelope {
            key: Some(key),
            addr: Some(addr),
            contents: contents,
            phantom: PhantomData,
        }
    }

    pub fn push_back(&mut self, msg: zmq::Message) {
        self.contents.push_back(msg);
    }
}

impl<E> Stream for Envelope<E> {
    type Item = zmq::Message;
    type Error = E;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        task::current().notify();
        match self.key.take() {
            Some(msg) => return Ok(Async::Ready(Some(msg))),
            None => (),
        }

        match self.addr.take() {
            Some(msg) => return Ok(Async::Ready(Some(msg))),
            None => (),
        }

        match self.contents.pop_front() {
            Some(msg) => Ok(Async::Ready(Some(msg))),
            None => Ok(Async::Ready(None)),
        }
    }
}

impl<E> Sink for Envelope<E> {
    type SinkItem = zmq::Message;
    type SinkError = E;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        self.push_back(item);

        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        Ok(Async::Ready(()))
    }
}
