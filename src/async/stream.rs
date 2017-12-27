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

use zmq;
use futures::{Async, Poll, Stream};
use futures::task;

#[derive(Clone)]
pub struct MsgStream {
    socket: Option<Rc<zmq::Socket>>,
}

impl MsgStream {
    pub fn new(sock: Rc<zmq::Socket>) -> Self {
        MsgStream { socket: Some(sock) }
    }

    fn next_message(&mut self) -> Result<Async<Option<zmq::Message>>, zmq::Error> {
        let socket = match self.socket.take() {
            Some(socket) => socket,
            None => return Ok(Async::Ready(None)),
        };

        let mut items = [socket.as_poll_item(zmq::POLLIN)];

        // Don't block waiting for an item to become ready
        zmq::poll(&mut items, 1)?;

        let mut msg = zmq::Message::new().unwrap();

        for item in items.iter() {
            if item.is_readable() {
                match socket.recv(&mut msg, zmq::DONTWAIT) {
                    Ok(_) => {
                        task::current().notify();
                        if msg.get_more() {
                            self.socket = Some(Rc::clone(&socket));
                        }
                        return Ok(Async::Ready(Some(msg)));
                    }
                    Err(zmq::Error::EAGAIN) => (),
                    Err(err) => {
                        return Err(err);
                    }
                }
            }
        }

        self.socket = Some(Rc::clone(&socket));

        task::current().notify();
        Ok(Async::NotReady)
    }
}

impl Stream for MsgStream {
    type Item = zmq::Message;
    type Error = zmq::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.next_message()
    }
}

#[derive(Clone)]
pub struct ZmqStream {
    socket: Rc<zmq::Socket>,
}

impl ZmqStream {
    pub fn new(sock: Rc<zmq::Socket>) -> Self {
        ZmqStream { socket: sock }
    }

    fn next_message(&self) -> Async<Option<MsgStream>> {
        Async::Ready(Some(MsgStream::new(Rc::clone(&self.socket))))
    }
}

impl Stream for ZmqStream {
    type Item = MsgStream;
    type Error = zmq::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        Ok(self.next_message())
    }
}

impl fmt::Debug for ZmqStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "ZmqStream")
    }
}
