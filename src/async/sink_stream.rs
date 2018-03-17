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

//! This module defines the `MultipartSinkStream` type. A wrapper around Sockets that implements
//! `futures::Sink` and `futures::Stream`.

use std::mem::swap;

use futures::{Async, Sink, Stream};
use futures::task::Context;
use tokio_file_unix::File;
use tokio::reactor::PollEvented2;
use zmq;

use async::sink::MultipartSink;
use async::stream::MultipartStream;
use error::Error;
use file::ZmqFile;
use message::Multipart;

/// The `MultipartSinkStream` handles sending and receiving streams of data to and from ZeroMQ
/// Sockets.
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
/// use futures::{FutureExt, Sink, Stream, StreamExt};
/// use tokio_zmq::{Error, Multipart, Socket};
///
/// fn get_sink_stream(socket: Socket) -> impl Sink<SinkItem = Multipart, SinkError = Error> + Stream<Item = Multipart, Error = Error>
/// {
///     socket.sink_stream()
/// }
///
/// fn main() {
///     let context = Arc::new(zmq::Context::new());
///     let socket = Socket::builder(context)
///         .bind("tcp://*:5575")
///         .build(zmq::REP)
///         .unwrap();
///
///     let sink_stream = get_sink_stream(socket);
///
///     let (sink, stream) = sink_stream.split();
///
///     // tokio::reactor::run2(stream.forward(sink));
/// }
/// ```
pub struct MultipartSinkStream {
    inner: SinkStreamState,
}

enum SinkStreamState {
    Sink(MultipartSink),
    Stream(MultipartStream),
    Both(
        MultipartSink,
        MultipartStream,
        zmq::Socket,
        PollEvented2<File<ZmqFile>>,
    ),
    Ready(zmq::Socket, PollEvented2<File<ZmqFile>>),
    Polling,
}

impl MultipartSinkStream {
    pub fn new(sock: zmq::Socket, file: PollEvented2<File<ZmqFile>>) -> Self {
        MultipartSinkStream {
            inner: SinkStreamState::Ready(sock, file),
        }
    }

    fn polling(&mut self) -> SinkStreamState {
        let mut state = SinkStreamState::Polling;

        swap(&mut self.inner, &mut state);

        state
    }

    fn poll_sink(
        &mut self,
        mut sink: MultipartSink,
        stream: Option<MultipartStream>,
        cx: &mut Context,
    ) -> Result<Async<()>, Error> {
        match sink.poll_flush(cx)? {
            Async::Ready(_) => match sink.take_socket() {
                Some((sock, file)) => {
                    debug!("Released sink");
                    match stream {
                        Some(mut stream) => {
                            stream.give_socket(sock, file);
                            self.inner = SinkStreamState::Stream(stream);
                        }
                        None => {
                            self.inner = SinkStreamState::Ready(sock, file);
                        }
                    }
                    Ok(Async::Ready(()))
                }
                None => Err(Error::Sink),
            },
            Async::Pending => {
                match stream {
                    Some(mut stream) => match sink.take_socket() {
                        Some((sock, file)) => {
                            self.inner = SinkStreamState::Both(sink, stream, sock, file);
                        }
                        None => {
                            return Err(Error::Sink);
                        }
                    },
                    None => {
                        self.inner = SinkStreamState::Sink(sink);
                    }
                }

                Ok(Async::Pending)
            }
        }
    }

    fn poll_stream(
        &mut self,
        mut stream: MultipartStream,
        sink: Option<MultipartSink>,
        cx: &mut Context,
    ) -> Result<Async<Option<Multipart>>, Error> {
        match stream.poll_next(cx)? {
            Async::Ready(item) => match stream.take_socket() {
                Some((sock, file)) => {
                    debug!("Released stream");
                    match sink {
                        Some(mut sink) => {
                            sink.give_socket(sock, file);
                            self.inner = SinkStreamState::Sink(sink);
                        }
                        None => {
                            self.inner = SinkStreamState::Ready(sock, file);
                        }
                    }
                    Ok(Async::Ready(item))
                }
                None => Err(Error::Stream),
            },
            Async::Pending => {
                match sink {
                    Some(mut sink) => match stream.take_socket() {
                        Some((sock, file)) => {
                            self.inner = SinkStreamState::Both(sink, stream, sock, file);
                        }
                        None => {
                            return Err(Error::Stream);
                        }
                    },
                    None => {
                        self.inner = SinkStreamState::Stream(stream);
                    }
                }

                Ok(Async::Pending)
            }
        }
    }
}

impl Sink for MultipartSinkStream {
    type SinkItem = Multipart;
    type SinkError = Error;

    fn start_send(&mut self, multipart: Self::SinkItem) -> Result<(), Self::SinkError> {
        debug!("Called start_send");
        match self.polling() {
            SinkStreamState::Ready(sock, file) => {
                let mut sink = MultipartSink::new(sock, file);
                sink.start_send(multipart)?;
                self.inner = SinkStreamState::Sink(sink);
                debug!("Created sink");
                Ok(())
            }
            SinkStreamState::Stream(mut stream) => match stream.take_socket() {
                Some((sock, file)) => {
                    let mut sink = MultipartSink::new(sock, file);
                    sink.start_send(multipart)?;
                    match sink.take_socket() {
                        Some((sock, file)) => {
                            self.inner = SinkStreamState::Both(sink, stream, sock, file);
                            debug!("Created sink");
                            Ok(())
                        }
                        None => Err(Error::Sink),
                    }
                }
                None => Err(Error::Sink),
            },
            _ => Err(Error::Sink),
        }
    }

    fn poll_ready(&mut self, cx: &mut Context) -> Result<Async<()>, Self::SinkError> {
        debug!("Called poll_ready");
        self.poll_flush(cx)
    }

    fn poll_flush(&mut self, cx: &mut Context) -> Result<Async<()>, Self::SinkError> {
        debug!("Called poll_flush");
        match self.polling() {
            SinkStreamState::Ready(sock, file) => {
                self.inner = SinkStreamState::Ready(sock, file);
                Ok(Async::Ready(()))
            }
            SinkStreamState::Sink(sink) => self.poll_sink(sink, None, cx),
            SinkStreamState::Stream(stream) => {
                self.inner = SinkStreamState::Stream(stream);
                Ok(Async::Ready(()))
            }
            SinkStreamState::Both(mut sink, stream, sock, file) => {
                sink.give_socket(sock, file);
                self.poll_sink(sink, Some(stream), cx)
            }
            SinkStreamState::Polling => Err(Error::Sink),
        }
    }

    fn poll_close(&mut self, cx: &mut Context) -> Result<Async<()>, Self::SinkError> {
        debug!("Called poll_close");
        self.poll_flush(cx)
    }
}

impl Stream for MultipartSinkStream {
    type Item = Multipart;
    type Error = Error;

    fn poll_next(&mut self, cx: &mut Context) -> Result<Async<Option<Multipart>>, Self::Error> {
        match self.polling() {
            SinkStreamState::Ready(sock, file) => {
                let stream = MultipartStream::new(sock, file);
                debug!("Created stream");
                self.poll_stream(stream, None, cx)
            }
            SinkStreamState::Sink(mut sink) => match sink.take_socket() {
                Some((sock, file)) => {
                    let stream = MultipartStream::new(sock, file);
                    debug!("Created stream");
                    self.poll_stream(stream, Some(sink), cx)
                }
                None => Err(Error::Stream),
            },
            SinkStreamState::Both(sink, mut stream, sock, file) => {
                stream.give_socket(sock, file);
                self.poll_stream(stream, Some(sink), cx)
            }
            SinkStreamState::Stream(stream) => self.poll_stream(stream, None, cx),
            SinkStreamState::Polling => Err(Error::Stream),
        }
    }
}
