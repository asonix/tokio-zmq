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

mod config;
pub mod types;

use std::rc::Rc;

use zmq;
use tokio_core::reactor::{Handle, PollEvented};
use tokio_file_unix::File;

use prelude::*;
use self::config::SocketBuilder;
use async::{ControlledStream, Multipart, MultipartRequest, MultipartResponse, MultipartSink,
            MultipartStream};
use error::Error;
use file::ZmqFile;

/// Defines the raw Socket type. This type should never be interacted with directly, except to
/// create new instances of wrapper types.
#[derive(Clone)]
pub struct Socket {
    // Reads and Writes data
    sock: Rc<zmq::Socket>,
    // So we can hand out files to streams and sinks
    file: Rc<PollEvented<File<ZmqFile>>>,
}

impl Socket {
    /// Start a new Socket Config builder
    pub fn create(ctx: Rc<zmq::Context>, handle: &Handle) -> SocketBuilder {
        SocketBuilder::new(ctx, handle)
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
    pub fn stream(&self) -> MultipartStream<DefaultEndHandler> {
        MultipartStream::new(Rc::clone(&self.sock), Rc::clone(&self.file))
    }

    /// Retrieve a stream that produces Multiparts. This stream has an end condition
    pub fn stream_with_end<E>(&self, end_handler: E) -> MultipartStream<E>
    where
        E: EndHandler,
    {
        MultipartStream::new_with_end(Rc::clone(&self.sock), Rc::clone(&self.file), end_handler)
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
#[derive(Clone)]
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
    pub fn stream<H>(&self, handler: H) -> ControlledStream<DefaultEndHandler, H>
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

    /// Retrieve a stream that produces multiparts, stopping when the should_stop control handler
    /// returns true, or when the should_stop end_handler returns true.
    pub fn stream_with_end<E, H>(&self, handler: H, end_handler: E) -> ControlledStream<E, H>
    where
        E: EndHandler,
        H: ControlHandler,
    {
        ControlledStream::new_with_end(
            Rc::clone(&self.stream_sock.sock),
            Rc::clone(&self.stream_sock.file),
            Rc::clone(&self.control_sock.sock),
            Rc::clone(&self.control_sock.file),
            handler,
            end_handler,
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
