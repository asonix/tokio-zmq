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
use futures::{Async, AsyncSink, Poll, Sink, StartSend};

#[derive(Clone)]
pub struct ZmqSink<E>
where
    E: From<zmq::Error>,
{
    socket: Rc<zmq::Socket>,
    phantom: PhantomData<E>,
}

impl<E> ZmqSink<E>
where
    E: From<zmq::Error>,
{
    pub fn new(sock: Rc<zmq::Socket>) -> Self {
        ZmqSink {
            socket: sock,
            phantom: PhantomData,
        }
    }

    fn send_message(&mut self, msg: zmq::Message) -> Result<AsyncSink<zmq::Message>, zmq::Error> {
        let mut items = [self.socket.as_poll_item(zmq::POLLOUT)];

        zmq::poll(&mut items, 1)?;

        for item in items.iter() {
            if item.is_writable() {
                match self.socket.send(&msg, zmq::DONTWAIT) {
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

    fn flush(&mut self) -> Result<Async<()>, zmq::Error> {
        let mut items = [self.socket.as_poll_item(zmq::POLLOUT)];

        zmq::poll(&mut items, 1).map(|_| Async::Ready(()))
    }
}

impl<E> Sink for ZmqSink<E>
where
    E: From<zmq::Error>,
{
    type SinkItem = zmq::Message;
    type SinkError = E;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        Ok(self.send_message(item)?)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        Ok(self.flush()?)
    }
}

impl<E> fmt::Debug for ZmqSink<E>
where
    E: From<zmq::Error>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "ZmqSink")
    }
}
