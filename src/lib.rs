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

#![feature(conservative_impl_trait)]

extern crate zmq;
#[macro_use]
extern crate zmq_futures_derive;
extern crate futures;

pub mod service;

pub mod async;
pub mod rep;
pub mod req;
pub mod zpub;
pub mod sub;
pub mod push;
pub mod pull;

pub use self::rep::Rep;
pub use self::req::Req;
pub use self::zpub::Pub;
pub use self::sub::Sub;
pub use self::push::Push;
pub use self::pull::Pull;
pub use self::service::{Handler, Runner};

use self::async::{ZmqRequest, ZmqResponse, ZmqSink, ZmqStream};
use futures::Stream;

use std::rc::Rc;

pub trait ZmqSocket {
    fn socket(&self) -> Rc<zmq::Socket>;
}

pub trait StreamSocket: ZmqSocket {
    fn recv(&self) -> async::ZmqResponse {
        ZmqResponse::new(self.socket())
    }

    fn stream<E>(&self) -> async::ZmqStream<E>
    where
        E: From<zmq::Error>,
    {
        ZmqStream::new(self.socket())
    }
}

pub trait SinkSocket: ZmqSocket {
    fn send(&self, msg: zmq::Message) -> ZmqRequest {
        ZmqRequest::new(self.socket(), msg)
    }

    fn sink<S, E>(&self) -> async::ZmqSink<S, E>
    where
        S: Stream<Item = zmq::Message, Error = E>,
        E: From<zmq::Error>,
    {
        ZmqSink::new(self.socket())
    }
}
