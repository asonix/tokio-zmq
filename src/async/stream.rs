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
use futures::{Async, Poll, Stream};
use futures::task;

#[derive(Clone)]
pub struct MsgStream<E>
where
    E: From<zmq::Error>,
{
    socket: Option<Rc<zmq::Socket>>,
    phantom: PhantomData<E>,
}

impl<E> MsgStream<E>
where
    E: From<zmq::Error>,
{
    pub fn new(sock: Rc<zmq::Socket>) -> Self {
        MsgStream {
            socket: Some(sock),
            phantom: PhantomData,
        }
    }

    fn next_message(&mut self) -> Result<Async<Option<zmq::Message>>, E> {
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
                        return Err(err.into());
                    }
                }
            }
        }

        self.socket = Some(Rc::clone(&socket));

        task::current().notify();
        Ok(Async::NotReady)
    }
}

impl<E> Stream for MsgStream<E>
where
    E: From<zmq::Error>,
{
    type Item = zmq::Message;
    type Error = E;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.next_message()
    }
}

#[derive(Clone)]
pub struct ZmqStream<E>
where
    E: From<zmq::Error>,
{
    socket: Rc<zmq::Socket>,
    phantom: PhantomData<E>,
}

impl<E> ZmqStream<E>
where
    E: From<zmq::Error>,
{
    pub fn new(sock: Rc<zmq::Socket>) -> Self {
        ZmqStream {
            socket: sock,
            phantom: PhantomData,
        }
    }

    fn next_message(&self) -> Async<Option<MsgStream<E>>> {
        Async::Ready(Some(MsgStream::new(Rc::clone(&self.socket))))
    }
}

impl<E> Stream for ZmqStream<E>
where
    E: From<zmq::Error>,
{
    type Item = MsgStream<E>;
    type Error = E;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        Ok(self.next_message())
    }
}

impl<E> fmt::Debug for ZmqStream<E>
where
    E: From<zmq::Error>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "ZmqStream")
    }
}
