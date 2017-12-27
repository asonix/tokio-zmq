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

pub struct ZmqResponse {
    socket: Rc<zmq::Socket>,
    msg: Option<zmq::Message>,
}

impl ZmqResponse {
    pub fn new(socket: Rc<zmq::Socket>, msg: zmq::Message) -> Self {
        ZmqResponse {
            socket: socket,
            msg: Some(msg),
        }
    }

    fn send(&mut self) -> bool {
        if let Some(msg) = self.msg.take() {
            let mut items = [self.socket.as_poll_item(zmq::POLLOUT)];

            match zmq::poll(&mut items, 1) {
                Ok(_) => (),
                Err(err) => {
                    println!("Error in poll: {}", err);
                    return false;
                }
            };

            for item in items.iter() {
                if item.is_writable() {
                    match self.socket.send(&msg, zmq::DONTWAIT) {
                        Ok(_) => {
                            return true;
                        }
                        Err(zmq::Error::EAGAIN) => {
                            println!("Socket full, wait");
                        }
                        Err(err) => {
                            println!("Error checking item: {}", err);
                        }
                    }

                    self.msg = Some(msg);

                    break;
                }
            }

            false
        } else {
            true
        }
    }

    fn receive(&mut self) -> Async<zmq::Message> {
        let mut items = [self.socket.as_poll_item(zmq::POLLIN)];

        // Don't block waiting for an item to become ready
        match zmq::poll(&mut items, 1) {
            Ok(_) => (),
            Err(_) => {
                return Async::NotReady;
            }
        };

        let mut msg = zmq::Message::new().unwrap();

        for item in items.iter() {
            if item.is_readable() {
                match self.socket.recv(&mut msg, zmq::DONTWAIT) {
                    Ok(_) => {
                        return Async::Ready(msg);
                    }
                    Err(zmq::Error::EAGAIN) => {
                        println!("Socket not ready, wait");
                    }
                    Err(err) => {
                        println!("Error checking item: {}", err);
                    }
                }
            }
        }

        task::current().notify();
        Async::NotReady
    }
}

impl Future for ZmqResponse {
    type Item = zmq::Message;
    type Error = ();


    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if self.send() {
            Ok(self.receive())
        } else {
            Ok(Async::NotReady)
        }
    }
}
