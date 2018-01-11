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

pub mod config;
pub mod types;

use std::rc::Rc;

use zmq;
use tokio_core::reactor::{Handle, PollEvented};
use tokio_file_unix::File;

use self::config::SocketBuilder;
use async::{MultipartRequest, MultipartResponse, MultipartSink, MultipartStream};
use message::Multipart;
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
    pub fn builder(ctx: Rc<zmq::Context>, handle: &Handle) -> SocketBuilder {
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
