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
use futures::{Async, Future, Poll, Stream};
use futures::stream::Collect;
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
        let event_count = zmq::poll(&mut items, 0)?;
        if event_count == 0 {
            task::current().notify();
            self.socket = Some(Rc::clone(&socket));
            return Ok(Async::NotReady);
        }

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

pub trait ControlHandler {
    type Request: From<Vec<zmq::Message>>;

    fn should_stop(&self, msg: Self::Request) -> bool;
}

pub struct ZmqControlledStream<C, E>
where
    C: ControlHandler,
    E: From<zmq::Error>,
{
    socket: Rc<zmq::Socket>,
    control: Rc<zmq::Socket>,
    control_handler: C,
    control_stream: Option<Collect<MsgStream<E>>>,
    phantom: PhantomData<E>,
}

impl<C, E> ZmqControlledStream<C, E>
where
    C: ControlHandler,
    E: From<zmq::Error>,
{
    pub fn new(sock: Rc<zmq::Socket>, control: Rc<zmq::Socket>, control_handler: C) -> Self {
        ZmqControlledStream {
            socket: sock,
            control: control,
            control_handler: control_handler,
            control_stream: None,
            phantom: PhantomData,
        }
    }

    fn next_message(&mut self) -> Result<Async<Option<MsgStream<E>>>, E> {
        let ctrl_stream = self.control_stream.take();

        if let Some(mut ctrl_stream) = ctrl_stream {
            match ctrl_stream.poll()? {
                Async::Ready(ctrl_vec) => {
                    if self.control_handler.should_stop(ctrl_vec.into()) {
                        return Ok(Async::Ready(None));
                    }
                }
                Async::NotReady => {
                    self.control_stream = Some(ctrl_stream);
                    task::current().notify();
                    return Ok(Async::NotReady);
                }
            }
        } else {
            let mut items = [self.control.as_poll_item(zmq::POLLIN)];

            if zmq::poll(&mut items, 0)? != 0 {
                let sock = Rc::clone(&self.control);
                let ctrl_stream = MsgStream::new(sock).collect();

                self.control_stream = Some(ctrl_stream);
                task::current().notify();
                return Ok(Async::NotReady);
            }
        }

        let mut items = [self.socket.as_poll_item(zmq::POLLIN)];
        if zmq::poll(&mut items, 1)? != 0 {
            Ok(Async::Ready(Some(MsgStream::new(Rc::clone(&self.socket)))))
        } else {
            task::current().notify();
            Ok(Async::NotReady)
        }
    }
}

impl<C, E> Stream for ZmqControlledStream<C, E>
where
    C: ControlHandler,
    E: From<zmq::Error>,
{
    type Item = MsgStream<E>;
    type Error = E;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.next_message()
    }
}

impl<C, E> fmt::Debug for ZmqControlledStream<C, E>
where
    C: ControlHandler,
    E: From<zmq::Error>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "ZmqControlledStream")
    }
}
