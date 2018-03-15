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

use std::mem::swap;
use std::time::Duration;

use futures::{Async, Future, Stream};
use futures::future::Either;
use futures::task::Context;
use tokio::reactor::PollEvented2;
use tokio_file_unix::File;
use tokio_timer::{Sleep, Timer};
use zmq;

use async::future::MultipartResponse;
use error::Error;
use file::ZmqFile;
use message::Multipart;
use prelude::{ControlHandler, EndHandler};

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
    inner: StreamState,
}

pub(crate) enum StreamState {
    Ready(zmq::Socket, PollEvented2<File<ZmqFile>>),
    Pending(MultipartResponse),
    Polling,
}

impl MultipartStream {
    pub fn new(sock: zmq::Socket, file: PollEvented2<File<ZmqFile>>) -> Self {
        MultipartStream {
            inner: StreamState::Ready(sock, file),
        }
    }

    pub(crate) fn take_socket(&mut self) -> Option<(zmq::Socket, PollEvented2<File<ZmqFile>>)> {
        match self.polling() {
            StreamState::Ready(sock, file) => Some((sock, file)),
            StreamState::Pending(mut response) => {
                let opt = response.take_socket();
                self.inner = StreamState::Pending(response);
                opt
            }
            StreamState::Polling => None,
        }
    }

    pub(crate) fn give_socket(&mut self, sock: zmq::Socket, file: PollEvented2<File<ZmqFile>>) {
        match self.polling() {
            StreamState::Pending(mut response) => {
                response.give_socket(sock, file);
                self.inner = StreamState::Pending(response);
            }
            _ => self.inner = StreamState::Ready(sock, file),
        }
    }

    pub(crate) fn polling(&mut self) -> StreamState {
        let mut state = StreamState::Polling;

        swap(&mut self.inner, &mut state);

        state
    }

    fn poll_response(
        &mut self,
        mut response: MultipartResponse,
        cx: &mut Context,
    ) -> Result<Async<Option<Multipart>>, Error> {
        match response.poll(cx)? {
            Async::Ready((item, sock, file)) => {
                self.inner = StreamState::Ready(sock, file);

                Ok(Async::Ready(Some(item)))
            }
            Async::Pending => {
                self.inner = StreamState::Pending(response);

                Ok(Async::Pending)
            }
        }
    }
}

impl Stream for MultipartStream {
    type Item = Multipart;
    type Error = Error;

    fn poll_next(&mut self, cx: &mut Context) -> Result<Async<Option<Multipart>>, Self::Error> {
        match self.polling() {
            StreamState::Ready(sock, file) => {
                let response = MultipartResponse::new(sock, file);

                self.poll_response(response, cx)
            }
            StreamState::Pending(response) => self.poll_response(response, cx),
            StreamState::Polling => Err(Error::Stream),
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
}

impl<E, S> Stream for EndingStream<E, S>
where
    E: EndHandler,
    S: Stream<Item = Multipart, Error = Error>,
{
    type Item = Multipart;
    type Error = Error;

    fn poll_next(&mut self, cx: &mut Context) -> Result<Async<Option<Multipart>>, Error> {
        let res = match self.stream.poll_next(cx)? {
            Async::Ready(Some(item)) => if self.end_handler.should_stop(&item) {
                Async::Ready(None)
            } else {
                Async::Ready(Some(item))
            },
            Async::Ready(None) => Async::Ready(None),
            Async::Pending => Async::Pending,
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
pub struct ControlledStream<H, S, T>
where
    H: ControlHandler,
    S: Stream<Item = Multipart, Error = Error>,
    T: Stream<Item = Multipart, Error = Error>,
{
    stream: T,
    control: S,
    handler: H,
}

impl<H, S, T> ControlledStream<H, S, T>
where
    H: ControlHandler,
    S: Stream<Item = Multipart, Error = Error>,
    T: Stream<Item = Multipart, Error = Error>,
{
    /// Create a new ControlledStream.
    ///
    /// This shouldn't be called directly. A socket wrapper type's `controlled` method, if present,
    /// will perform the required actions to create and encapsulate this type.
    pub fn new(stream: T, control: S, handler: H) -> ControlledStream<H, S, T> {
        ControlledStream {
            stream,
            control,
            handler,
        }
    }
}

impl<H, S, T> Stream for ControlledStream<H, S, T>
where
    H: ControlHandler,
    S: Stream<Item = Multipart, Error = Error>,
    T: Stream<Item = Multipart, Error = Error>,
{
    type Item = Multipart;
    type Error = Error;

    /// Poll the control stream, if it isn't ready, poll the producing stream
    ///
    /// If the control stream is ready, but has ended, stop the producting stream.
    /// If the control stream is ready with a Multipart, use the `ControlHandler`
    /// to determine if the producting stream should be stopped.
    fn poll_next(&mut self, cx: &mut Context) -> Result<Async<Option<Multipart>>, Error> {
        let stop = match self.control.poll_next(cx)? {
            Async::Pending => false,
            Async::Ready(None) => true,
            Async::Ready(Some(multipart)) => self.handler.should_stop(multipart),
        };

        if stop {
            Ok(Async::Ready(None))
        } else {
            self.stream.poll_next(cx)
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

    fn poll_next(&mut self, cx: &mut Context) -> Result<Async<Option<Self::Item>>, Self::Error> {
        if let Async::Ready(_) = self.timeout.poll(cx)? {
            self.timeout = self.timer.sleep(self.duration);

            return Ok(Async::Ready(Some(Either::Right(Timeout))));
        }

        let res = match self.stream.poll_next(cx)? {
            Async::Ready(Some(item)) => Async::Ready(Some(Either::Left(item))),
            Async::Ready(None) => Async::Ready(None),
            Async::Pending => Async::Pending,
        };

        Ok(res)
    }
}
