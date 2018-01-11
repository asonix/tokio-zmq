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

use std::rc::Rc;
use std::time::Duration;

use futures::{Async, Future, Poll, Stream};
use futures::future::Either;
use tokio_core::reactor::PollEvented;
use tokio_file_unix::File;
use tokio_timer::{Sleep, Timer};
use zmq;

use async::future::MultipartResponse;
use error::Error;
use file::ZmqFile;
use message::Multipart;
use prelude::{ControlHandler, EndHandler, StreamSocket};

/// The `MultipartStream` Sink handles receiving streams of data from ZeroMQ Sockets.
///
/// You shouldn't ever need to manually create one. Here's how to get one from a 'raw' `Socket`'
/// type.
///
/// ### Example
/// ```rust
/// #![feature(conservative_impl_trait)]
///
/// extern crate zmq;
/// extern crate futures;
/// extern crate tokio_core;
/// extern crate tokio_zmq;
///
/// use std::rc::Rc;
///
/// use futures::Stream;
/// use tokio_core::reactor::Core;
/// use tokio_zmq::async::MultipartStream;
/// use tokio_zmq::{Error, Multipart, Socket};
///
/// fn get_stream(socket: Socket) -> impl Stream<Item = Multipart, Error = Error> {
///     socket.stream().and_then(|multipart| {
///         // handle multipart
///         Ok(multipart)
///     })
/// }
///
/// fn main() {
///     let core = Core::new().unwrap();
///     let context = Rc::new(zmq::Context::new());
///     let socket = Socket::builder(context, &core.handle())
///         .connect("tcp://localhost:5568")
///         .filter(b"")
///         .build(zmq::SUB)
///         .unwrap();
///     get_stream(socket);
/// }
/// ```
pub struct MultipartStream {
    // To build a multipart
    response: Option<MultipartResponse>,
    // To read data
    sock: Rc<zmq::Socket>,
    // Handles notifications to/from the event loop
    file: Rc<PollEvented<File<ZmqFile>>>,
}

impl MultipartStream {
    pub fn new(sock: Rc<zmq::Socket>, file: Rc<PollEvented<File<ZmqFile>>>) -> Self {
        MultipartStream {
            response: None,
            sock: sock,
            file: file,
        }
    }

    /// Add a timeout to this stream
    pub fn timeout(self, duration: Duration) -> TimeoutStream<Self> {
        TimeoutStream::new(self, duration)
    }

    /// Add an EndHandler to this stream
    pub fn with_end<E>(self, end_handler: E) -> EndingStream<E, Self>
    where
        E: EndHandler,
    {
        EndingStream::new(self, end_handler)
    }

    /// Add a control stream to this stream
    pub fn controlled<A, H>(self, control: A, handler: H) -> ControlledStream<H, Self>
    where
        A: StreamSocket,
        H: ControlHandler,
    {
        ControlledStream::new(self, control, handler)
    }

    fn poll_response(&mut self, mut response: MultipartResponse) -> Poll<Option<Multipart>, Error> {
        match response.poll()? {
            Async::Ready(item) => Ok(Async::Ready(Some(item))),
            Async::NotReady => {
                self.response = Some(response);
                self.file.need_read();
                Ok(Async::NotReady)
            }
        }
    }
}

impl Stream for MultipartStream {
    type Item = Multipart;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Multipart>, Error> {
        if let Some(response) = self.response.take() {
            self.poll_response(response)
        } else {
            let response = MultipartResponse::new(Rc::clone(&self.sock), Rc::clone(&self.file));
            self.poll_response(response)
        }
    }
}

/// A stream that ends when the `EndHandler`'s `should_stop` method returns True
pub struct EndingStream<E, S>
where
    E: EndHandler,
    S: Stream<Item = Multipart, Error = Error>,
{
    stream: S,
    // To handle stopping
    end_handler: E,
}

impl<E, S> EndingStream<E, S>
where
    E: EndHandler,
    S: Stream<Item = Multipart, Error = Error>,
{
    /// Wrap a stream with an EndHandler
    pub fn new(stream: S, end_handler: E) -> Self
    where
        E: EndHandler,
    {
        EndingStream {
            stream,
            end_handler,
        }
    }

    /// Add a timeout to this stream
    pub fn timeout(self, duration: Duration) -> TimeoutStream<Self> {
        TimeoutStream::new(self, duration)
    }

    /// Add a control stream to this stream
    pub fn controlled<A, H>(self, control: A, handler: H) -> ControlledStream<H, Self>
    where
        A: StreamSocket,
        H: ControlHandler,
    {
        ControlledStream::new(self, control, handler)
    }
}

impl<E, S> Stream for EndingStream<E, S>
where
    E: EndHandler,
    S: Stream<Item = Multipart, Error = Error>,
{
    type Item = Multipart;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Multipart>, Error> {
        let res = match self.stream.poll()? {
            Async::Ready(Some(item)) => if self.end_handler.should_stop(&item) {
                Async::Ready(None)
            } else {
                Async::Ready(Some(item))
            },
            Async::Ready(None) => Async::Ready(None),
            Async::NotReady => Async::NotReady,
        };

        Ok(res)
    }
}

/// `ControlledStream`s are used when you want a stream of multiparts, but you want to be able to
/// turn it off.
///
/// It contains a handler that implements the `ControlHandler` trait. This trait contains a single
/// method `should_stop`, that determines whether or not the given stream should stop producing
/// values.
pub struct ControlledStream<H, S>
where
    H: ControlHandler,
    S: Stream<Item = Multipart, Error = Error>,
{
    stream: S,
    control: MultipartStream,
    handler: H,
}

impl<H, S> ControlledStream<H, S>
where
    H: ControlHandler,
    S: Stream<Item = Multipart, Error = Error>,
{
    /// Create a new ControlledStream.
    ///
    /// This shouldn't be called directly. A socket wrapper type's `controlled` method, if present,
    /// will perform the required actions to create and encapsulate this type.
    pub fn new<A>(stream: S, control_sock: A, handler: H) -> ControlledStream<H, S>
    where
        A: StreamSocket,
    {
        let control = control_sock.into_socket().stream();

        ControlledStream {
            stream,
            control,
            handler,
        }
    }

    /// Add a timeout to this stream
    pub fn timeout(self, duration: Duration) -> TimeoutStream<Self> {
        TimeoutStream::new(self, duration)
    }

    /// Add an EndHandler to this stream
    pub fn with_end<E>(self, end_handler: E) -> EndingStream<E, Self>
    where
        E: EndHandler,
    {
        EndingStream::new(self, end_handler)
    }
}

impl<H, S> Stream for ControlledStream<H, S>
where
    H: ControlHandler,
    S: Stream<Item = Multipart, Error = Error>,
{
    type Item = Multipart;
    type Error = Error;

    /// Poll the control stream, if it isn't ready, poll the producing stream
    ///
    /// If the control stream is ready, but has ended, stop the producting stream.
    /// If the control stream is ready with a Multipart, use the `ControlHandler`
    /// to determine if the producting stream should be stopped.
    fn poll(&mut self) -> Poll<Option<Multipart>, Error> {
        let stop = match self.control.poll()? {
            Async::NotReady => false,
            Async::Ready(None) => true,
            Async::Ready(Some(multipart)) => self.handler.should_stop(multipart),
        };

        if stop {
            Ok(Async::Ready(None))
        } else {
            self.stream.poll()
        }
    }
}

/// An empty type to represent a timeout event
pub struct Timeout;

/// A stream that provides either an `Item` or a `Timeout`
///
/// This is different from `tokio_timer::TimeoutStream<T>`, since that stream errors on timeout.
pub struct TimeoutStream<S>
where
    S: Stream,
{
    stream: S,
    duration: Duration,
    timer: Timer,
    timeout: Sleep,
}

impl<S> TimeoutStream<S>
where
    S: Stream<Error = Error>,
{
    /// Add a timeout to a stream
    pub fn new(stream: S, duration: Duration) -> Self {
        let timer = Timer::default();
        let timeout = timer.sleep(duration);

        TimeoutStream {
            stream,
            duration,
            timer,
            timeout,
        }
    }
}

impl<S> Stream for TimeoutStream<S>
where
    S: Stream<Error = Error>,
{
    type Item = Either<S::Item, Timeout>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if let Async::Ready(_) = self.timeout.poll()? {
            self.timeout = self.timer.sleep(self.duration);

            return Ok(Async::Ready(Some(Either::B(Timeout))));
        }

        let res = match self.stream.poll()? {
            Async::Ready(Some(item)) => Async::Ready(Some(Either::A(item))),
            Async::Ready(None) => Async::Ready(None),
            Async::NotReady => Async::NotReady,
        };

        Ok(res)
    }
}
