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

use zmq;
use tokio_core::reactor::PollEvented;
use tokio_file_unix::File;
use futures::{Async, Future, Poll, Stream};

use async::future::MultipartResponse;
use error::Error;
use super::Multipart;
use file::ZmqFile;

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
/// use tokio_zmq::async::{Multipart, MultipartStream};
/// use tokio_zmq::{Error, Socket};
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
///     let socket = Socket::new(context, core.handle())
///         .connect("tcp://localhost:5568")
///         .filter(b"")
///         .build(zmq::SUB)
///         .unwrap();
///     get_stream(socket);
/// }
/// ```
pub struct MultipartStream {
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

    fn poll_response(&mut self, mut response: MultipartResponse) -> Poll<Option<Multipart>, Error> {
        match response.poll()? {
            Async::Ready(item) => Ok(Async::Ready(Some(item))),
            Async::NotReady => {
                self.response = Some(response);
                Ok(Async::NotReady)
            }
        }
    }
}

impl Stream for MultipartStream {
    type Item = Multipart;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Multipart>, Error> {
        debug!("MultipartStream: In poll");
        if let Some(response) = self.response.take() {
            self.poll_response(response)
        } else {
            let response = MultipartResponse::new(Rc::clone(&self.sock), Rc::clone(&self.file));
            self.poll_response(response)
        }
    }
}

/// The `ControlHandler` trait is used to impose stopping rules for streams that otherwise would
/// continue to create multiparts.
pub trait ControlHandler {
    /// `should_stop` determines whether or not a `ControlledStream` should stop producing values.
    ///
    /// It accepts a Multipart as input. This Multipart comes from the ControlledStream's
    /// associated control MultipartStream.
    fn should_stop(&self, multipart: Multipart) -> bool;
}

/// `ControlledStream`s are used when you want a stream of multiparts, but you want to be able to
/// turn it off.
///
/// It contains a handler that implements the `ControlHandler` trait. This trait contains a single
/// method `should_stop`, that determines whether or not the given stream should stop producing
/// values.
pub struct ControlledStream<H>
where
    H: ControlHandler,
{
    stream: MultipartStream,
    control: MultipartStream,
    handler: H,
}

impl<H> ControlledStream<H>
where
    H: ControlHandler,
{
    /// Create a new ControlledStream.
    ///
    /// This shouldn't be called directly. A socket wrapper type's `controlled` method, if present,
    /// will perform the required actions to create and encapsulate this type.
    pub fn new(
        sock: Rc<zmq::Socket>,
        file: Rc<PollEvented<File<ZmqFile>>>,
        control_sock: Rc<zmq::Socket>,
        control_file: Rc<PollEvented<File<ZmqFile>>>,
        handler: H,
    ) -> Self {
        ControlledStream {
            stream: MultipartStream::new(sock, file),
            control: MultipartStream::new(control_sock, control_file),
            handler: handler,
        }
    }
}

impl<H> Stream for ControlledStream<H>
where
    H: ControlHandler,
{
    type Item = Multipart;
    type Error = Error;

    /// Poll the control stream, if it isn't ready, poll the producing stream
    ///
    /// If the control stream is ready, but has ended, stop the producting stream.
    /// If the control stream is ready with a Multipart, use the `ControlHandler` to
    /// determine if the producting stream should be stopped.
    fn poll(&mut self) -> Poll<Option<Multipart>, Error> {
        debug!("ControlledStream: in poll");
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
