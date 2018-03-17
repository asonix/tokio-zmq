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

//! This module defines the `MultipartSink` type. A wrapper around Sockets that implements
//! `futures::Sink`.

use std::mem::swap;

use zmq;
use tokio::reactor::PollEvented2;
use futures::task::Context;
use tokio_file_unix::File;
use futures::{Async, Future, Sink};

use message::Multipart;
use async::future::MultipartRequest;
use error::Error;
use file::ZmqFile;

/// The `MultipartSink` Sink handles sending streams of data to ZeroMQ Sockets.
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
/// extern crate tokio;
/// extern crate tokio_zmq;
///
/// use std::sync::Arc;
///
/// use futures::{FutureExt, Sink, SinkExt};
/// use tokio_zmq::{Error, Multipart, Socket};
///
/// fn get_sink(socket: Socket) -> impl Sink<SinkItem = Multipart, SinkError = Error> {
///     socket.sink()
/// }
///
/// fn main() {
///     let context = Arc::new(zmq::Context::new());
///     let socket = Socket::builder(context)
///         .bind("tcp://*:5568")
///         .build(zmq::PUB)
///         .unwrap();
///     let sink = get_sink(socket);
///
///     let msg = zmq::Message::from_slice(b"Some message");
///
///     // tokio::reactor::run2(sink.send(msg.into())).unwrap();
/// }
/// ```
pub struct MultipartSink {
    inner: SinkState,
}

pub(crate) enum SinkState {
    Ready(zmq::Socket, PollEvented2<File<ZmqFile>>),
    Pending(MultipartRequest<(zmq::Socket, PollEvented2<File<ZmqFile>>)>),
    Polling,
}

impl MultipartSink {
    pub fn new(sock: zmq::Socket, file: PollEvented2<File<ZmqFile>>) -> Self {
        MultipartSink {
            inner: SinkState::Ready(sock, file),
        }
    }

    pub(crate) fn take_socket(&mut self) -> Option<(zmq::Socket, PollEvented2<File<ZmqFile>>)> {
        match self.polling() {
            SinkState::Ready(sock, file) => Some((sock, file)),
            SinkState::Pending(mut request) => {
                let opt = request.take_socket();
                self.inner = SinkState::Pending(request);
                opt
            }
            SinkState::Polling => None,
        }
    }

    pub(crate) fn give_socket(&mut self, sock: zmq::Socket, file: PollEvented2<File<ZmqFile>>) {
        match self.polling() {
            SinkState::Pending(mut request) => {
                request.give_socket(sock, file);
                self.inner = SinkState::Pending(request);
            }
            _ => self.inner = SinkState::Ready(sock, file),
        }
    }

    pub(crate) fn polling(&mut self) -> SinkState {
        let mut state = SinkState::Polling;

        swap(&mut state, &mut self.inner);

        state
    }

    fn poll_request(
        &mut self,
        mut request: MultipartRequest<(zmq::Socket, PollEvented2<File<ZmqFile>>)>,
        cx: &mut Context,
    ) -> Result<Async<()>, Error> {
        match request.poll(cx)? {
            Async::Ready((sock, file)) => {
                self.inner = SinkState::Ready(sock, file);

                Ok(Async::Ready(()))
            }
            Async::Pending => {
                self.inner = SinkState::Pending(request);

                Ok(Async::Pending)
            }
        }
    }

    fn make_request(&mut self, multipart: Multipart) -> Result<(), Error> {
        match self.polling() {
            SinkState::Ready(sock, file) => {
                self.inner = SinkState::Pending(MultipartRequest::new(sock, file, multipart));
                Ok(())
            }
            _ => Err(Error::Sink),
        }
    }
}

impl Sink for MultipartSink {
    type SinkItem = Multipart;
    type SinkError = Error;

    fn start_send(&mut self, multipart: Self::SinkItem) -> Result<(), Self::SinkError> {
        self.make_request(multipart)?;

        Ok(())
    }

    fn poll_ready(&mut self, cx: &mut Context) -> Result<Async<()>, Self::SinkError> {
        self.poll_flush(cx)
    }

    fn poll_flush(&mut self, cx: &mut Context) -> Result<Async<()>, Self::SinkError> {
        match self.polling() {
            SinkState::Pending(request) => self.poll_request(request, cx),
            SinkState::Ready(sock, file) => {
                self.inner = SinkState::Ready(sock, file);
                Ok(Async::Ready(()))
            }
            SinkState::Polling => Err(Error::Sink),
        }
    }

    fn poll_close(&mut self, cx: &mut Context) -> Result<Async<()>, Self::SinkError> {
        self.poll_flush(cx)
    }
}
