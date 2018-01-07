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
//! `futures::Stream`.

use std::marker::PhantomData;
use std::rc::Rc;

use zmq;
use tokio_core::reactor::PollEvented;
use tokio_file_unix::File;
use futures::{Async, AsyncSink, Future, Poll, Sink, StartSend};

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
/// extern crate tokio_core;
/// extern crate tokio_zmq;
///
/// use std::rc::Rc;
///
/// use futures::{Future, Sink};
/// use tokio_core::reactor::Core;
/// use tokio_zmq::async::{MultipartStream};
/// use tokio_zmq::{Error, Multipart, Socket};
///
/// fn get_sink(socket: Socket) -> impl Sink<SinkItem = Multipart, SinkError = Error> {
///     socket.sink()
/// }
///
/// fn main() {
///     let mut core = Core::new().unwrap();
///     let context = Rc::new(zmq::Context::new());
///     let socket = Socket::create(context, &core.handle())
///         .bind("tcp://*:5568")
///         .build(zmq::PUB)
///         .unwrap();
///     let sink = get_sink(socket);
///
///     let msg = zmq::Message::from_slice(b"Some message").unwrap();
///
///     core.run(sink.send(msg.into())).unwrap();
/// }
/// ```
pub struct MultipartSink<E>
where
    E: From<Error>,
{
    request: Option<MultipartRequest>,
    sock: Rc<zmq::Socket>,
    // Handles notifications to/from the event loop
    file: Rc<PollEvented<File<ZmqFile>>>,
    phantom: PhantomData<E>,
}

impl<E> MultipartSink<E>
where
    E: From<Error>,
{
    pub fn new(sock: Rc<zmq::Socket>, file: Rc<PollEvented<File<ZmqFile>>>) -> Self {
        MultipartSink {
            request: None,
            sock: sock,
            file: file,
            phantom: PhantomData,
        }
    }

    fn poll_request(&mut self, mut request: MultipartRequest) -> Poll<(), E> {
        match request.poll()? {
            Async::Ready(()) => Ok(Async::Ready(())),
            Async::NotReady => {
                self.request = Some(request);
                Ok(Async::NotReady)
            }
        }
    }

    fn make_request(&mut self, multipart: Multipart) {
        let sock = Rc::clone(&self.sock);
        let file = Rc::clone(&self.file);

        self.request = Some(MultipartRequest::new(sock, file, multipart));
    }
}

impl<E> Sink for MultipartSink<E>
where
    E: From<Error>,
{
    type SinkItem = Multipart;
    type SinkError = E;

    fn start_send(
        &mut self,
        multipart: Self::SinkItem,
    ) -> StartSend<Self::SinkItem, Self::SinkError> {
        debug!("MultipartSink: start_send");
        if let Some(request) = self.request.take() {
            match self.poll_request(request)? {
                Async::Ready(()) => {
                    self.make_request(multipart);
                    self.file.need_write();

                    Ok(AsyncSink::Ready)
                }
                Async::NotReady => Ok(AsyncSink::NotReady(multipart)),
            }
        } else {
            self.make_request(multipart);
            self.file.need_write();

            Ok(AsyncSink::Ready)
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        debug!("MultipartSink: poll_complete");
        if let Some(request) = self.request.take() {
            self.poll_request(request)
        } else {
            Ok(Async::Ready(()))
        }
    }
}
