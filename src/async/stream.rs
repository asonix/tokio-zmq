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
pub struct ZmqStream {
    socket: Rc<zmq::Socket>,
}

impl ZmqStream {
    pub fn new(sock: Rc<zmq::Socket>) -> Self {
        ZmqStream { socket: sock }
    }

    fn next_message(&self) -> Result<Async<Option<zmq::Message>>, zmq::Error> {
        let mut items = [self.socket.as_poll_item(zmq::POLLIN)];

        // Don't block waiting for an item to become ready
        zmq::poll(&mut items, 1)?;

        let mut msg = zmq::Message::new().unwrap();

        for item in items.iter() {
            if item.is_readable() {
                match self.socket.recv(&mut msg, zmq::DONTWAIT) {
                    Ok(_) => {
                        task::current().notify();
                        return Ok(Async::Ready(Some(msg)));
                    }
                    Err(zmq::Error::EAGAIN) => (),
                    Err(err) => {
                        return Err(err);
                    }
                }
            }
        }

        task::current().notify();
        Ok(Async::NotReady)
    }
}

impl Stream for ZmqStream {
    type Item = zmq::Message;
    type Error = zmq::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.next_message()
    }
}

impl fmt::Debug for ZmqStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "ZmqStream")
    }
}
