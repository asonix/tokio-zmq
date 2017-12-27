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

use zmq;

use futures::{Async, Future, Poll};
use futures::task;

pub struct ZmqRequest {
    socket: Rc<zmq::Socket>,
    msg: Option<zmq::Message>,
}

impl ZmqRequest {
    pub fn new(
        socket: Rc<zmq::Socket>,
        msg: zmq::Message,
    ) -> impl Future<Item = (), Error = zmq::Error> {
        ZmqRequest {
            socket: socket,
            msg: Some(msg),
        }
    }

    fn send(&mut self) -> Result<Async<()>, zmq::Error> {
        let msg = match self.msg.take() {
            Some(msg) => msg,
            None => return Ok(Async::Ready(())),
        };

        let mut items = [self.socket.as_poll_item(zmq::POLLOUT)];

        zmq::poll(&mut items, 1)?;

        for item in items.iter() {
            if item.is_writable() {
                match self.socket.send(&msg, zmq::DONTWAIT) {
                    Ok(_) => {
                        return Ok(Async::Ready(()));
                    }
                    Err(zmq::Error::EAGAIN) => (),
                    Err(err) => {
                        return Err(err);
                    }
                }

                self.msg = Some(msg);

                break;
            }
        }

        task::current().notify();
        Ok(Async::NotReady)
    }
}

impl Future for ZmqRequest {
    type Item = ();
    type Error = zmq::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.send()
    }
}

pub struct ZmqResponse {
    socket: Rc<zmq::Socket>,
}

impl ZmqResponse {
    pub fn new(
        socket: Rc<zmq::Socket>,
        msg: zmq::Message,
    ) -> impl Future<Item = zmq::Message, Error = zmq::Error> {
        ZmqRequest::new(Rc::clone(&socket), msg).and_then(|_| ZmqResponse { socket: socket })
    }

    fn receive(&mut self) -> Result<Async<zmq::Message>, zmq::Error> {
        let mut items = [self.socket.as_poll_item(zmq::POLLIN)];

        // Don't block waiting for an item to become ready
        zmq::poll(&mut items, 1)?;

        let mut msg = zmq::Message::new().unwrap();

        for item in items.iter() {
            if item.is_readable() {
                match self.socket.recv(&mut msg, zmq::DONTWAIT) {
                    Ok(_) => {
                        return Ok(Async::Ready(msg));
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

impl Future for ZmqResponse {
    type Item = zmq::Message;
    type Error = zmq::Error;


    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.receive()
    }
}
