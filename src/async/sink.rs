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
use std::fmt;
use std::marker::PhantomData;

use zmq;
use futures::{Async, AsyncSink, Poll, Sink, StartSend, Stream};

#[derive(PartialEq)]
enum MsgPlace {
    NthMsg,
    LastMsg,
}

pub struct ZmqSink<S, E>
where
    S: Stream<Item = zmq::Message, Error = E>,
    E: From<zmq::Error>,
{
    socket: Rc<zmq::Socket>,
    current_msg: Option<zmq::Message>,
    next_msg: Option<zmq::Message>,
    stream: Option<S>,
    phantom: PhantomData<E>,
}

impl<S, E> ZmqSink<S, E>
where
    S: Stream<Item = zmq::Message, Error = E>,
    E: From<zmq::Error>,
{
    pub fn new(sock: Rc<zmq::Socket>) -> Self {
        ZmqSink {
            socket: sock,
            current_msg: None,
            next_msg: None,
            stream: None,
            phantom: PhantomData,
        }
    }

    fn send_message(
        &mut self,
        msg: zmq::Message,
        place: MsgPlace,
    ) -> Result<AsyncSink<zmq::Message>, zmq::Error> {
        let mut items = [self.socket.as_poll_item(zmq::POLLOUT)];

        zmq::poll(&mut items, 1)?;

        let flags = zmq::DONTWAIT |
            if place == MsgPlace::LastMsg {
                0
            } else {
                zmq::SNDMORE
            };

        for item in items.iter() {
            if item.is_writable() {
                match self.socket.send(&msg, flags) {
                    Ok(_) => {
                        return Ok(AsyncSink::Ready);
                    }
                    Err(zmq::Error::EAGAIN) => (),
                    Err(err) => {
                        return Err(err);
                    }
                }

                break;
            }
        }

        Ok(AsyncSink::NotReady(msg))
    }

    fn flush(&mut self) -> Result<Async<()>, E> {
        if let Some(current_msg) = self.current_msg.take() {
            let msg_place = if self.next_msg.is_some() {
                MsgPlace::NthMsg
            } else {
                MsgPlace::LastMsg
            };

            match self.send_message(current_msg, msg_place)? {
                AsyncSink::Ready => (),
                AsyncSink::NotReady(current_msg) => {
                    self.current_msg = Some(current_msg);
                    return Ok(Async::NotReady);
                }
            }
        }

        if self.next_msg.is_some() {
            self.current_msg = self.next_msg.take();
        }

        let item = {
            let stream = match self.stream {
                Some(ref mut stream) => stream,
                None => {
                    return if self.current_msg.is_some() {
                        Ok(Async::NotReady)
                    } else {
                        Ok(Async::Ready(()))
                    };
                }
            };

            match stream.poll()? {
                Async::Ready(item) => item,
                Async::NotReady => return Ok(Async::NotReady),
            }
        };

        match item {
            Some(msg) => {
                self.next_msg = Some(msg);
                Ok(Async::NotReady)
            }
            None => {
                self.stream = None;
                Ok(Async::NotReady)
            }
        }
    }
}

impl<S, E> Sink for ZmqSink<S, E>
where
    S: Stream<Item = zmq::Message, Error = E>,
    E: From<zmq::Error>,
{
    type SinkItem = S;
    type SinkError = E;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        if self.stream.is_none() && self.next_msg.is_none() && self.current_msg.is_none() {
            self.stream = Some(item);

            Ok(AsyncSink::Ready)
        } else {
            match self.flush()? {
                Async::Ready(()) => Ok(AsyncSink::Ready),
                Async::NotReady => Ok(AsyncSink::NotReady(item)),
            }
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        self.flush()
    }
}

impl<S, E> fmt::Debug for ZmqSink<S, E>
where
    S: Stream<Item = zmq::Message, Error = E>,
    E: From<zmq::Error>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "ZmqSink")
    }
}
