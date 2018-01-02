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

//! This module contains useful traits and types for working with ZeroMQ Sockets.

use std::rc::Rc;

use zmq;
use tokio_core::reactor::{Handle, PollEvented};
use tokio_file_unix::File;

mod config;
mod rep;
mod req;
mod zpub;
mod sub;
mod push;
mod pull;
mod xpub;
mod xsub;
mod pair;
mod dealer;
mod router;

pub use self::rep::{Rep, RepControlled};
pub use self::req::Req;
pub use self::zpub::Pub;
pub use self::sub::{Sub, SubControlled};
pub use self::push::Push;
pub use self::pull::{Pull, PullControlled};
pub use self::xpub::{Xpub, XpubControlled};
pub use self::xsub::{Xsub, XsubControlled};
pub use self::pair::{Pair, PairControlled};
pub use self::dealer::{Dealer, DealerControlled};
pub use self::router::{Router, RouterControlled};

use self::config::SockConfigStart;
use async::{ControlledStream, ControlHandler, Multipart, MultipartRequest, MultipartResponse,
            MultipartSink, MultipartStream};
use error::Error;
use file::ZmqFile;

/// The AsSocket trait is implemented for all wrapper types. This makes implementing other traits a
/// matter of saying a given type implements them.
pub trait AsSocket {
    /// Any type implementing AsSocket must have a way of returning a reference to a Socket.
    fn socket(&self) -> &Socket;

    /// Any type implementing AsSocket must have a way of consuming itself and returning a socket.
    fn into_socket(self) -> Socket;
}

/// Analogous to the AsSocket trait, but for Controlled sockets.
pub trait AsControlledSocket {
    /// Any implementing type must have a method of getting a reference to the inner
    /// `ControlledSocket`.
    fn socket(&self) -> &ControlledSocket;
}

/// This trait is used for types wrapping `ControlledSocket`s. It depends on the type implementing
/// AsControlledSocket, which is analogous to the `AsSocket` trait's `socket(&self)` method.
pub trait ControlledStreamSocket<H>: AsControlledSocket
where
    H: ControlHandler,
{
    /// Receive a single multipart message from the socket.
    fn recv(&self) -> MultipartResponse {
        self.socket().recv()
    }

    /// Receive a stream of multipart messages from the socket.
    fn stream(&self, handler: H) -> ControlledStream<H> {
        self.socket().stream(handler)
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

/// This trait provides the basic Stream support for ZeroMQ Sockets. It depends on AsSocket, but
/// provides implementations for `sink` and `recv`.
pub trait StreamSocket: AsSocket {
    /// Receive a single multipart message from the socket.
    ///
    /// ### Example, using the Rep wrapper type
    /// ```rust
    /// #![feature(conservative_impl_trait)]
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
    ///     let rep: Rep = Socket::new(context, core.handle())
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
    /// #![feature(conservative_impl_trait)]
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
    ///     let sub: Sub = Socket::new(context, core.handle())
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
    fn stream(&self) -> MultipartStream {
        self.socket().stream()
    }
}

/// This trait provides the basic Sink support for ZeroMQ Sockets. It depends on AsSocket and
/// provides the `send` and `sink` methods.
pub trait SinkSocket: AsSocket {
    /// Send a single multipart message to the socket.
    ///
    /// ### Example, using a Pub wrapper type
    /// ```rust
    /// #![feature(conservative_impl_trait)]
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
    ///     let zpub: Pub = Socket::new(context, core.handle())
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
    /// #![feature(conservative_impl_trait)]
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
    ///     let zpub: Pub = Socket::new(context, core.handle())
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
    /// #![feature(conservative_impl_trait)]
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
    ///     let zpub: Pub = Socket::new(context, core.handle())
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
    /// #![feature(conservative_impl_trait)]
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
    ///     let rep: Rep = Socket::new(context, core.handle())
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

/// Defines the raw Socket type. This type should never be interacted with directly, except to
/// create new instances of wrapper types.
pub struct Socket {
    // Reads and Writes data
    sock: Rc<zmq::Socket>,
    // So we can hand out files to streams and sinks
    file: Rc<PollEvented<File<ZmqFile>>>,
}

impl Socket {
    /// Start a new Socket Config builder
    pub fn new(ctx: Rc<zmq::Context>, handle: Handle) -> SockConfigStart {
        SockConfigStart::new(ctx, handle)
    }

    /// Retrieve a Reference-Counted Pointer to self's socket.
    pub fn inner_sock(&self) -> Rc<zmq::Socket> {
        Rc::clone(&self.sock)
    }

    /// Retrieve a Reference-Counted Pointer to self's file.
    pub fn inner_file(&self) -> Rc<PollEvented<File<ZmqFile>>> {
        Rc::clone(&self.file)
    }

    /// Create a new socket from a given Sock and File
    ///
    /// This assumes that `sock` is already configured properly. Please don't call this directly
    /// unless you know what you're doing.
    pub fn from_sock_and_file(sock: Rc<zmq::Socket>, file: Rc<PollEvented<File<ZmqFile>>>) -> Self {
        Socket { sock, file }
    }

    /// Create a ControlledSocket from this and a controller socket
    ///
    /// The resulting ControlledSocket receives it's main data from the current socket, and control
    /// commands from the control socket.
    pub fn controlled<S>(self, control: S) -> ControlledSocket
    where
        S: StreamSocket,
    {
        ControlledSocket {
            stream_sock: self,
            control_sock: control.into_socket(),
        }
    }

    /// Retrieve a Sink that consumes Multiparts, sending them to the socket
    pub fn sink<E>(&self) -> MultipartSink<E>
    where
        E: From<Error>,
    {
        MultipartSink::new(Rc::clone(&self.sock), Rc::clone(&self.file))
    }

    /// Retrieve a Stream that produces Multiparts, getting them from the socket
    pub fn stream(&self) -> MultipartStream {
        MultipartStream::new(Rc::clone(&self.sock), Rc::clone(&self.file))
    }

    /// Retrieve a Future that consumes a multipart, sending it to the socket
    pub fn send(&self, multipart: Multipart) -> MultipartRequest {
        MultipartRequest::new(Rc::clone(&self.sock), Rc::clone(&self.file), multipart)
    }

    /// Retrieve a Future that produces a multipart, getting it fromthe socket
    pub fn recv(&self) -> MultipartResponse {
        MultipartResponse::new(Rc::clone(&self.sock), Rc::clone(&self.file))
    }
}

/// Defines a raw `ControlledSocket` type
///
/// Controlled sockets are useful for being able to stop streams. They shouldn't be created
/// directly, but through a wrapper type's `controlled` method.
pub struct ControlledSocket {
    stream_sock: Socket,
    control_sock: Socket,
}

impl ControlledSocket {
    /// Retrieve a sink that consumes multiparts, sending them to the socket.
    pub fn sink<E>(&self) -> MultipartSink<E>
    where
        E: From<Error>,
    {
        self.stream_sock.sink()
    }

    /// Retrieve a stream that produces multiparts, stopping when the should_stop control handler
    /// returns true.
    pub fn stream<H>(&self, handler: H) -> ControlledStream<H>
    where
        H: ControlHandler,
    {
        ControlledStream::new(
            Rc::clone(&self.stream_sock.sock),
            Rc::clone(&self.stream_sock.file),
            Rc::clone(&self.control_sock.sock),
            Rc::clone(&self.control_sock.file),
            handler,
        )
    }

    /// Sends a single multipart to the socket
    pub fn send(&self, multipart: Multipart) -> MultipartRequest {
        self.stream_sock.send(multipart)
    }

    /// Receives a single multipart from the socket
    pub fn recv(&self) -> MultipartResponse {
        self.stream_sock.recv()
    }
}
