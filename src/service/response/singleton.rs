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

use zmq;
use futures::{Async, Poll, Stream, task};

pub struct Singleton<E> {
    inner: Option<zmq::Message>,
    phantom: PhantomData<E>,
}

impl<E> From<zmq::Message> for Singleton<E> {
    fn from(msg: zmq::Message) -> Self {
        Singleton {
            inner: Some(msg),
            phantom: PhantomData,
        }
    }
}

impl<E> Stream for Singleton<E> {
    type Item = zmq::Message;
    type Error = E;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        task::current().notify();
        Ok(Async::Ready(self.inner.take()))
    }
}
