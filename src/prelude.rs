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

use std::time::Duration;

use futures_core::Stream;
use tokio::reactor::PollEvented2;
use tokio_file_unix::File;
use zmq;

use async::{ControlledStream, EndingStream, MultipartRequest, MultipartResponse, MultipartSink,
            MultipartSinkStream, MultipartStream, TimeoutStream};
use error::Error;
use file::ZmqFile;
use message::Multipart;
use socket::Socket;

/* ----------------------------------TYPES----------------------------------- */

/* ----------------------------------TRAITS---------------------------------- */

/// The `AsSocket` trait is implemented for all wrapper types. This makes implementing other traits a
/// matter of saying a given type implements them.
pub trait AsSocket: From<(zmq::Socket, PollEvented2<File<ZmqFile>>)> + Sized {
    /// Any type implementing `AsSocket` must have a way of returning a reference to a Socket.
    fn socket(self) -> Socket;
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

/// This trait provides the basic Stream support for ZeroMQ Sockets. It depends on `AsSocket`, but
/// provides implementations for `sink` and `recv`.
pub trait StreamSocket: AsSocket {
    /// Receive a single multipart message from the socket.
    ///
    /// ### Example, using the Rep wrapper type
    /// ```rust
    /// #![feature(try_from)]
    ///
    /// extern crate futures_util;
    /// extern crate tokio;
    /// extern crate tokio_zmq;
    /// extern crate zmq;
    ///
    /// use std::convert::TryInto;
    /// use std::sync::Arc;
    ///
    /// use futures_util::FutureExt;
    /// use tokio_zmq::prelude::*;
    /// use tokio_zmq::async::MultipartStream;
    /// use tokio_zmq::{Error, Multipart, Rep, Socket};
    ///
    /// fn main() {
    ///     let context = Arc::new(zmq::Context::new());
    ///     let rep: Rep = Socket::builder(context)
    ///         .connect("tcp://localhost:5568")
    ///         .try_into()
    ///         .unwrap();
    ///
    ///     let fut = rep.recv().and_then(|(multipart, _)| {
    ///         for msg in &multipart {
    ///             if let Some(msg) = msg.as_str() {
    ///                 println!("Message: {}", msg);
    ///             }
    ///         }
    ///         Ok(multipart)
    ///     });
    ///
    ///     // tokio::runtime::run2(fut.map(|_| ()).or_else(|e| {
    ///     //     println!("Error: {}", e);
    ///     //     Ok(())
    ///     // }));
    ///     # let _ = fut;
    /// }
    /// ```
    fn recv(self) -> MultipartResponse<Self> {
        self.socket().recv()
    }

    /// Receive a stream of multipart messages from the socket.
    ///
    /// ### Example, using a Sub wrapper type
    /// ```rust
    /// #![feature(try_from)]
    ///
    /// extern crate zmq;
    /// extern crate futures_util;
    /// extern crate tokio;
    /// extern crate tokio_zmq;
    ///
    /// use std::convert::TryInto;
    /// use std::sync::Arc;
    ///
    /// use futures_util::{FutureExt, StreamExt};
    /// use tokio_zmq::prelude::*;
    /// use tokio_zmq::async::{MultipartStream};
    /// use tokio_zmq::{Error, Multipart, Socket, Sub};
    ///
    /// fn main() {
    ///     let context = Arc::new(zmq::Context::new());
    ///     let sub: Sub = Socket::builder(context)
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
    ///     // tokio::runtime::run2(fut.map(|_| ()).or_else(|e| {
    ///     //     println!("Error: {}", e);
    ///     //     Ok(())
    ///     // }));
    /// }
    /// ```
    fn stream(self) -> MultipartStream {
        self.socket().stream()
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
    /// extern crate futures_util;
    /// extern crate tokio;
    /// extern crate tokio_zmq;
    ///
    /// use std::convert::TryInto;
    /// use std::sync::Arc;
    ///
    /// use futures_util::FutureExt;
    /// use tokio_zmq::prelude::*;
    /// use tokio_zmq::async::MultipartStream;
    /// use tokio_zmq::{Error, Pub, Socket};
    ///
    /// fn main() {
    ///     let context = Arc::new(zmq::Context::new());
    ///     let zpub: Pub = Socket::builder(context)
    ///         .connect("tcp://localhost:5569")
    ///         .try_into()
    ///         .unwrap();
    ///
    ///     let msg = zmq::Message::from_slice(b"Hello").unwrap();
    ///
    ///     let fut = zpub.send(msg.into());
    ///
    ///     // tokio::runtime::run2(fut.map(|_| ()).or_else(|e| {
    ///     //     println!("Error: {}", e);
    ///     //     Ok(())
    ///     // }));
    /// }
    /// ```
    fn send(self, multipart: Multipart) -> MultipartRequest<Self> {
        self.socket().send(multipart)
    }

    /// Send a stream of multipart messages to the socket.
    ///
    /// ### Example, using a Pub wrapper type
    /// ```rust
    /// #![feature(try_from)]
    ///
    /// extern crate zmq;
    /// extern crate futures_util;
    /// extern crate tokio;
    /// extern crate tokio_zmq;
    ///
    /// use std::convert::TryInto;
    /// use std::sync::Arc;
    ///
    /// use futures_util::{FutureExt, StreamExt};
    /// use futures_util::stream::iter_ok;
    /// use tokio_zmq::prelude::*;
    /// use tokio_zmq::async::MultipartStream;
    /// use tokio_zmq::{Error, Multipart, Pub, Socket};
    ///
    /// fn main() {
    ///     let context = Arc::new(zmq::Context::new());
    ///     let zpub: Pub = Socket::builder(context)
    ///         .connect("tcp://localhost:5570")
    ///         .try_into()
    ///         .unwrap();
    ///
    ///     let fut = iter_ok(0..5)
    ///         .and_then(|i| {
    ///             let msg = zmq::Message::from_slice(format!("i: {}", i).as_bytes())?;
    ///             Ok(msg.into()) as Result<Multipart, Error>
    ///         })
    ///         .forward(zpub.sink());
    ///
    ///     // tokio::runtime::run2(fut.map(|_| ()).or_else(|e| {
    ///     //     println!("Error: {}", e);
    ///     //     Ok(())
    ///     // }));
    /// }
    /// ```
    fn sink(self) -> MultipartSink {
        self.socket().sink()
    }
}

/// This trait is provided for sockets that implement both Sync and Stream
pub trait SinkStreamSocket: AsSocket {
    /// Retrieve a structure that implements both Sync and Stream.
    ///
    /// ### Example, using a Rep wrapper type
    /// ```rust
    /// #![feature(try_from)]
    ///
    /// extern crate futures_util;
    /// extern crate tokio_zmq;
    /// extern crate zmq;
    ///
    /// use std::convert::TryInto;
    /// use std::sync::Arc;
    ///
    /// use futures_util::{FutureExt, StreamExt};
    /// use tokio_zmq::prelude::*;
    /// use tokio_zmq::{Socket, Rep};
    ///
    /// fn main() {
    ///     let ctx = Arc::new(zmq::Context::new());
    ///     let rep: Rep = Socket::builder(ctx)
    ///         .bind("tcp://*:5571")
    ///         .try_into()
    ///         .unwrap();
    ///
    ///     let (sink, stream) = rep.sink_stream().split();
    ///
    ///     let fut = stream.forward(sink);
    ///
    ///     // tokio::reactor::run2(fut.map(|_| ()).or_else(|e| {
    ///     //     println!("Error: {}", e);
    ///     //     Ok(())
    ///     // }));
    /// }
    /// ```
    fn sink_stream(self) -> MultipartSinkStream;
}

/// This trait is provided to allow for ending a stream based on a Multipart message it receives.
pub trait WithEndHandler: Stream<Item = Multipart, Error = Error> + Sized {
    /// Add an EndHandler to a stream.
    ///
    /// ### Example, using a Sub wrapper type
    /// ```rust
    /// #![feature(try_from)]
    ///
    /// extern crate futures_util;
    /// extern crate tokio_zmq;
    /// extern crate zmq;
    ///
    /// use std::convert::TryInto;
    /// use std::sync::Arc;
    ///
    /// use futures_util::{FutureExt, StreamExt};
    /// use tokio_zmq::prelude::*;
    /// use tokio_zmq::{Socket, Sub, Multipart};
    ///
    /// struct End(u32);
    ///
    /// impl EndHandler for End {
    ///     fn should_stop(&mut self, multipart: &Multipart) -> bool {
    ///         self.0 += 1;
    ///
    ///         self.0 > 30
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let ctx = Arc::new(zmq::Context::new());
    ///     let sub: Sub = Socket::builder(ctx)
    ///         .bind("tcp://*:5571")
    ///         .filter(b"")
    ///         .try_into()
    ///         .unwrap();
    ///
    ///     let fut = sub.stream().with_end_handler(End(0));
    ///
    ///     // tokio::reactor::run2(fut.map(|_| ()).or_else(|e| {
    ///     //     println!("Error: {}", e);
    ///     //     Ok(())
    ///     // }));
    /// }
    /// ```
    fn with_end_handler<E>(self, end_handler: E) -> EndingStream<E, Self>
    where
        E: EndHandler;
}

/// This trait is implemented by all Streams with Item = Multipart and Error = Error, it provides
/// the ability to control when the stream stops based on the content of another stream.
pub trait Controllable: Stream<Item = Multipart, Error = Error> + Sized {
    /// Add a controller stream to a given stream. This allows the controller stream to decide when
    /// the controlled stream should stop.
    ///
    /// ### Example, using a controlled Pull wrapper type and a controller Sub wrapper type
    /// ```rust
    /// #![feature(try_from)]
    ///
    /// extern crate futures_util;
    /// extern crate tokio_zmq;
    /// extern crate zmq;
    ///
    /// use std::convert::TryInto;
    /// use std::sync::Arc;
    ///
    /// use futures_util::{FutureExt, StreamExt};
    /// use tokio_zmq::prelude::*;
    /// use tokio_zmq::{Socket, Pull, Sub, Multipart};
    ///
    /// struct End;
    ///
    /// impl ControlHandler for End {
    ///     fn should_stop(&mut self, _: Multipart) -> bool {
    ///         true
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let ctx = Arc::new(zmq::Context::new());
    ///     let pull: Pull = Socket::builder(Arc::clone(&ctx))
    ///         .bind("tcp://*:5572")
    ///         .try_into()
    ///         .unwrap();
    ///
    ///     let sub: Sub = Socket::builder(ctx)
    ///         .bind("tcp://*:5573")
    ///         .filter(b"")
    ///         .try_into()
    ///         .unwrap();
    ///
    ///     let fut = pull.stream().controlled(sub.stream(), End);
    ///
    ///     // tokio::reactor::run2(fut.map(|_| ()).or_else(|e| {
    ///     //     println!("Error: {}", e);
    ///     //     Ok(())
    ///     // }));
    /// }
    /// ```
    fn controlled<H, S>(self, control_stream: S, handler: H) -> ControlledStream<H, S, Self>
    where
        H: ControlHandler,
        S: Stream<Item = Multipart, Error = Error>;
}

/// This trait allows adding a timeout to any stream with Error = Error.
pub trait WithTimeout: Stream<Error = Error> + Sized {
    /// Add a timeout to a given stream.
    ///
    /// ### Example, using a Pull wrapper type
    /// ```rust
    /// #![feature(try_from)]
    ///
    /// extern crate futures_util;
    /// extern crate tokio_zmq;
    /// extern crate zmq;
    ///
    /// use std::convert::TryInto;
    /// use std::sync::Arc;
    /// use std::time::Duration;
    ///
    /// use futures_util::{FutureExt, StreamExt};
    /// use tokio_zmq::prelude::*;
    /// use tokio_zmq::{Socket, Pull, Multipart};
    ///
    /// fn main() {
    ///     let ctx = Arc::new(zmq::Context::new());
    ///     let pull: Pull = Socket::builder(ctx)
    ///         .bind("tcp://*:5574")
    ///         .try_into()
    ///         .unwrap();
    ///
    ///     // Receive a Timeout after 30 seconds if the stream hasn't produced a value
    ///     let fut = pull.stream().timeout(Duration::from_secs(30));
    ///
    ///     // tokio::reactor::run2(fut.map(|_| ()).or_else(|e| {
    ///     //     println!("Error: {}", e);
    ///     //     Ok(())
    ///     // }));
    /// }
    /// ```
    fn timeout(self, duration: Duration) -> TimeoutStream<Self>;
}

/* ----------------------------------impls----------------------------------- */

impl<T> SinkStreamSocket for T
where
    T: StreamSocket + SinkSocket,
{
    fn sink_stream(self) -> MultipartSinkStream {
        self.socket().sink_stream()
    }
}

impl<T> WithEndHandler for T
where
    T: Stream<Item = Multipart, Error = Error>,
{
    fn with_end_handler<E>(self, end_handler: E) -> EndingStream<E, Self>
    where
        E: EndHandler,
    {
        EndingStream::new(self, end_handler)
    }
}

impl<T> Controllable for T
where
    T: Stream<Item = Multipart, Error = Error>,
{
    fn controlled<H, S>(self, control_stream: S, handler: H) -> ControlledStream<H, S, Self>
    where
        H: ControlHandler,
        S: Stream<Item = Multipart, Error = Error>,
    {
        ControlledStream::new(self, control_stream, handler)
    }
}

impl<T> WithTimeout for T
where
    T: Stream<Error = Error>,
{
    fn timeout(self, duration: Duration) -> TimeoutStream<Self> {
        TimeoutStream::new(self, duration)
    }
}
