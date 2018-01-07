/*
 * This file is part of Tokio ZMQ.
 *
 * Copyright © 2017 Riley Trautman
 *
 * Tokio ZMQ is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Tokio ZMQ is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Tokio ZMQ.  If not, see <http://www.gnu.org/licenses/>.
 */

//! Provide useful types and traits for working with Tokio ZMQ.

use socket::{ControlledSocket, Socket};
use async::{ControlledStream, MultipartRequest, MultipartResponse, MultipartSink,
            MultipartStream};
use message::Multipart;
use error::Error;

/* ----------------------------------TYPES----------------------------------- */

/// The Default `EndHandler` struct. This type is never instantiated, but instead is used in place
/// of a generic type for the default `MultipartStream` type.
///
/// It's `should_stop` method always returns false, although it should never be called.
pub struct DefaultEndHandler;

impl EndHandler for DefaultEndHandler {
    fn should_stop(&mut self, _: &Multipart) -> bool {
        false
    }
}

/* ----------------------------------TRAITS---------------------------------- */


/// The `AsSocket` trait is implemented for all wrapper types. This makes implementing other traits a
/// matter of saying a given type implements them.
pub trait AsSocket {
    /// Any type implementing `AsSocket` must have a way of returning a reference to a Socket.
    fn socket(&self) -> &Socket;

    /// Any type implementing `AsSocket` must have a way of consuming itself and returning a socket.
    fn into_socket(self) -> Socket;
}

/// Analogous to the `AsSocket` trait, but for Controlled sockets.
pub trait AsControlledSocket {
    /// Any implementing type must have a method of getting a reference to the inner
    /// `ControlledSocket`.
    fn socket(&self) -> &ControlledSocket;
}

/// This trait is used for types wrapping `ControlledSocket`s. It depends on the type implementing
/// `AsControlledSocket`, which is analogous to the `AsSocket` trait's `socket(&self)` method.
pub trait ControlledStreamSocket: AsControlledSocket {
    /// Receive a single multipart message from the socket.
    fn recv(&self) -> MultipartResponse {
        self.socket().recv()
    }

    /// Receive a stream of multipart messages from the socket.
    fn stream<H>(&self, handler: H) -> ControlledStream<DefaultEndHandler, H>
    where
        H: ControlHandler,
    {
        self.socket().stream(handler)
    }

    /// Receive a stream of multipart messages from the socket, ending when handler or
    /// end_handler's should_stop return true.
    fn stream_with_end<E, H>(&self, handler: H, end_handler: E) -> ControlledStream<E, H>
    where
        H: ControlHandler,
        E: EndHandler,
    {
        self.socket().stream_with_end(handler, end_handler)
    }
}

/// In addition to having Streams, some sockets also have Sinks. Every controlled socket has a
/// stream, but to give Sink functionality to those that have Sinks as well, this trait is
/// implemented.
pub trait ControlledSinkSocket: AsControlledSocket {
    /// Send a single multipart message to the socket.
    ///
    /// For example usage of `send`, see `SinkSocket`'s send definition.
    fn send(&self, multipart: Multipart) -> MultipartRequest {
        self.socket().send(multipart)
    }

    /// Get a sink to send a stream of multipart messages to the socket.
    ///
    /// For example usage of `sink`, see `SinkSocket`'s sink definition.
    fn sink<E>(&self) -> MultipartSink<E>
    where
        E: From<Error>,
    {
        self.socket().sink()
    }
}

/// The `ControlHandler` trait is used to impose stopping rules for streams that otherwise would
/// continue to create multiparts.
pub trait ControlHandler {
    /// `should_stop` determines whether or not a `ControlledStream` should stop producing values.
    ///
    /// It accepts a Multipart as input. This Multipart comes from the ControlledStream's
    /// associated control MultipartStream. If you want to have a socket that stops based on the
    /// content of a message it receives, see the `EndHandler` trait.
    fn should_stop(&mut self, multipart: Multipart) -> bool;
}

/// The `EndHandler` trait is used to impose stopping rules for streams that otherwise would
/// continue to create multiparts.
pub trait EndHandler {
    /// `should_stop` determines whether or not a `StreamSocket` should stop producing values.
    ///
    /// This method should be used if the stop signal sent to a given socket will be in-line with
    /// the rest of the messages that socket receives. If you want to have a socket controlled by
    /// another socket, see the `ControlHandler` trait.
    fn should_stop(&mut self, multipart: &Multipart) -> bool;
}

/// The `IntoControlledSocket` trait is used on Streams to allow the stream to be controlled by
/// messages from another socket.
pub trait IntoControlledSocket: StreamSocket {
    type Controlled: AsControlledSocket;

    /// Construct a controlled version of the given socket.
    fn controlled<S>(self, control: S) -> Self::Controlled
    where
        S: StreamSocket;
}

/// This trait provides the basic Stream support for ZeroMQ Sockets. It depends on `AsSocket`, but
/// provides implementations for `sink` and `recv`.
pub trait StreamSocket: AsSocket {
    /// Receive a single multipart message from the socket.
    ///
    /// ### Example, using the Rep wrapper type
    /// ```rust
    /// #![feature(try_from)]
    ///
    /// extern crate zmq;
    /// extern crate futures;
    /// extern crate tokio_core;
    /// extern crate tokio_zmq;
    ///
    /// use std::rc::Rc;
    /// use std::convert::TryInto;
    ///
    /// use futures::Future;
    /// use tokio_core::reactor::Core;
    /// use tokio_zmq::prelude::*;
    /// use tokio_zmq::async::{Multipart, MultipartStream};
    /// use tokio_zmq::{Error, Rep, Socket};
    ///
    /// fn main() {
    ///     let core = Core::new().unwrap();
    ///     let context = Rc::new(zmq::Context::new());
    ///     let rep: Rep = Socket::create(context, &core.handle())
    ///         .connect("tcp://localhost:5568")
    ///         .try_into()
    ///         .unwrap();
    ///
    ///     let fut = rep.recv().and_then(|multipart| {
    ///         for msg in &multipart {
    ///             if let Some(msg) = msg.as_str() {
    ///                 println!("Message: {}", msg);
    ///             }
    ///         }
    ///         Ok(multipart)
    ///     });
    ///
    ///     // core.run(fut).unwrap();
    ///     # let _ = fut;
    /// }
    fn recv(&self) -> MultipartResponse {
        self.socket().recv()
    }

    /// Receive a stream of multipart messages from the socket.
    ///
    /// ### Example, using a Sub wrapper type
    /// ```rust
    /// #![feature(try_from)]
    ///
    /// extern crate zmq;
    /// extern crate futures;
    /// extern crate tokio_core;
    /// extern crate tokio_zmq;
    ///
    /// use std::rc::Rc;
    /// use std::convert::TryInto;
    ///
    /// use futures::Stream;
    /// use tokio_core::reactor::Core;
    /// use tokio_zmq::prelude::*;
    /// use tokio_zmq::async::{Multipart, MultipartStream};
    /// use tokio_zmq::{Error, Socket, Sub};
    ///
    /// fn main() {
    ///     let core = Core::new().unwrap();
    ///     let context = Rc::new(zmq::Context::new());
    ///     let sub: Sub = Socket::create(context, &core.handle())
    ///         .connect("tcp://localhost:5569")
    ///         .filter(b"")
    ///         .try_into()
    ///         .unwrap();
    ///
    ///     let fut = sub.stream().for_each(|multipart| {
    ///         for msg in multipart {
    ///             if let Some(msg) = msg.as_str() {
    ///                 println!("Message: {}", msg);
    ///             }
    ///         }
    ///         Ok(())
    ///     });
    ///
    ///     // core.run(fut).unwrap();
    ///     # let _ = fut;
    /// }
    fn stream(&self) -> MultipartStream<DefaultEndHandler> {
        self.socket().stream()
    }

    /// Receive a stream of multipart messages from the socket ending when end_handler's
    /// should_stop returns true.
    ///
    /// ### Example, using a Sub wrapper type
    /// ```rust
    /// #![feature(try_from)]
    ///
    /// extern crate zmq;
    /// extern crate futures;
    /// extern crate tokio_core;
    /// extern crate tokio_zmq;
    ///
    /// use std::rc::Rc;
    /// use std::convert::TryInto;
    ///
    /// use futures::Stream;
    /// use tokio_core::reactor::Core;
    /// use tokio_zmq::prelude::*;
    /// use tokio_zmq::async::{Multipart, MultipartStream};
    /// use tokio_zmq::{Error, Socket, Sub};
    ///
    /// struct Stop {
    ///     count: usize,
    /// }
    ///
    /// impl Stop {
    ///     fn new() -> Self {
    ///         Stop {
    ///             count: 0,
    ///         }
    ///     }
    /// }
    ///
    /// impl EndHandler for Stop {
    ///     fn should_stop(&mut self, multipart: &Multipart) -> bool {
    ///         if self.count < 10 {
    ///             self.count += 1;
    ///
    ///             false
    ///         } else {
    ///             true
    ///         }
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let core = Core::new().unwrap();
    ///     let context = Rc::new(zmq::Context::new());
    ///     let sub: Sub = Socket::create(context, &core.handle())
    ///         .connect("tcp://localhost:5569")
    ///         .filter(b"")
    ///         .try_into()
    ///         .unwrap();
    ///
    ///     let fut = sub.stream_with_end(Stop::new()).for_each(|multipart| {
    ///         for msg in multipart {
    ///             if let Some(msg) = msg.as_str() {
    ///                 println!("Message: {}", msg);
    ///             }
    ///         }
    ///         Ok(())
    ///     });
    ///
    ///     // core.run(fut).unwrap();
    ///     # let _ = fut;
    /// }
    fn stream_with_end<E>(&self, end_handler: E) -> MultipartStream<E>
    where
        E: EndHandler,
    {
        self.socket().stream_with_end(end_handler)
    }
}

/// This trait provides the basic Sink support for ZeroMQ Sockets. It depends on `AsSocket` and
/// provides the `send` and `sink` methods.
pub trait SinkSocket: AsSocket {
    /// Send a single multipart message to the socket.
    ///
    /// ### Example, using a Pub wrapper type
    /// ```rust
    /// #![feature(try_from)]
    ///
    /// extern crate zmq;
    /// extern crate futures;
    /// extern crate tokio_core;
    /// extern crate tokio_zmq;
    ///
    /// use std::rc::Rc;
    /// use std::convert::TryInto;
    /// use std::collections::VecDeque;
    ///
    /// use tokio_core::reactor::Core;
    /// use tokio_zmq::prelude::*;
    /// use tokio_zmq::async::{Multipart, MultipartStream};
    /// use tokio_zmq::{Error, Pub, Socket};
    ///
    /// fn main() {
    ///     let mut core = Core::new().unwrap();
    ///     let context = Rc::new(zmq::Context::new());
    ///     let zpub: Pub = Socket::create(context, &core.handle())
    ///         .connect("tcp://localhost:5569")
    ///         .try_into()
    ///         .unwrap();
    ///
    ///     let msg = zmq::Message::from_slice(b"Hello").unwrap();
    ///     let mut multipart = VecDeque::new();
    ///     multipart.push_back(msg);
    ///
    ///     let fut = zpub.send(multipart);
    ///
    ///     core.run(fut).unwrap();
    /// }
    fn send(&self, multipart: Multipart) -> MultipartRequest {
        self.socket().send(multipart)
    }

    /// Send a stream of multipart messages to the socket.
    ///
    /// ### Example, using a Pub wrapper type
    /// ```rust
    /// #![feature(try_from)]
    ///
    /// extern crate zmq;
    /// extern crate futures;
    /// extern crate tokio_core;
    /// extern crate tokio_zmq;
    ///
    /// use std::rc::Rc;
    /// use std::convert::TryInto;
    /// use std::collections::VecDeque;
    ///
    /// use futures::Stream;
    /// use futures::stream::iter_ok;
    /// use tokio_core::reactor::Core;
    /// use tokio_zmq::prelude::*;
    /// use tokio_zmq::async::{Multipart, MultipartStream};
    /// use tokio_zmq::{Error, Pub, Socket};
    ///
    /// fn main() {
    ///     let mut core = Core::new().unwrap();
    ///     let context = Rc::new(zmq::Context::new());
    ///     let zpub: Pub = Socket::create(context, &core.handle())
    ///         .connect("tcp://localhost:5570")
    ///         .try_into()
    ///         .unwrap();
    ///
    ///     let fut = iter_ok(0..5)
    ///         .and_then(|i| {
    ///             let msg = zmq::Message::from_slice(format!("i: {}", i).as_bytes())?;
    ///             let mut multipart = VecDeque::new();
    ///             multipart.push_back(msg);
    ///             Ok(multipart) as Result<Multipart, Error>
    ///         })
    ///         .forward(zpub.sink::<Error>());
    ///
    ///     core.run(fut).unwrap();
    /// }
    fn sink<E>(&self) -> MultipartSink<E>
    where
        E: From<Error>,
    {
        self.socket().sink()
    }
}

/// This trait is used for socket types that don't really fit in the context of `Stream` or `Sink`.
/// Typically interaction with these sockets is one-off. This trait provides implementations for
/// `send` and `recv`.
pub trait FutureSocket: AsSocket {
    /// Send a single multipart message to the socket.
    ///
    /// ### Example, using a Pub wrapper type
    /// ```rust
    /// #![feature(try_from)]
    ///
    /// extern crate zmq;
    /// extern crate futures;
    /// extern crate tokio_core;
    /// extern crate tokio_zmq;
    ///
    /// use std::rc::Rc;
    /// use std::convert::TryInto;
    /// use std::collections::VecDeque;
    ///
    /// use tokio_core::reactor::Core;
    /// use tokio_zmq::prelude::*;
    /// use tokio_zmq::async::{Multipart, MultipartStream};
    /// use tokio_zmq::{Error, Pub, Socket};
    ///
    /// fn main() {
    ///     let mut core = Core::new().unwrap();
    ///     let context = Rc::new(zmq::Context::new());
    ///     let zpub: Pub = Socket::create(context, &core.handle())
    ///         .connect("tcp://localhost:5569")
    ///         .try_into()
    ///         .unwrap();
    ///
    ///     let msg = zmq::Message::from_slice(b"Hello").unwrap();
    ///     let mut multipart = VecDeque::new();
    ///     multipart.push_back(msg);
    ///
    ///     let fut = zpub.send(multipart);
    ///
    ///     core.run(fut).unwrap();
    /// }
    fn send(&self, multipart: Multipart) -> MultipartRequest {
        self.socket().send(multipart)
    }

    /// Receive a single multipart message from the socket.
    ///
    /// ### Example, using the Rep wrapper type
    /// ```rust
    /// #![feature(try_from)]
    ///
    /// extern crate zmq;
    /// extern crate futures;
    /// extern crate tokio_core;
    /// extern crate tokio_zmq;
    ///
    /// use std::rc::Rc;
    /// use std::convert::TryInto;
    ///
    /// use futures::Future;
    /// use tokio_core::reactor::Core;
    /// use tokio_zmq::prelude::*;
    /// use tokio_zmq::async::{Multipart, MultipartStream};
    /// use tokio_zmq::{Error, Rep, Socket};
    ///
    /// fn main() {
    ///     let core = Core::new().unwrap();
    ///     let context = Rc::new(zmq::Context::new());
    ///     let rep: Rep = Socket::create(context, &core.handle())
    ///         .connect("tcp://localhost:5568")
    ///         .try_into()
    ///         .unwrap();
    ///
    ///     let fut = rep.recv().and_then(|multipart| {
    ///         for msg in &multipart {
    ///             if let Some(msg) = msg.as_str() {
    ///                 println!("Message: {}", msg);
    ///             }
    ///         }
    ///         Ok(multipart)
    ///     });
    ///
    ///     // core.run(fut).unwrap();
    ///     # let _ = fut;
    /// }
    fn recv(&self) -> MultipartResponse {
        self.socket().recv()
    }
}
