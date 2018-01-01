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
use tokio_core::reactor::{Handle, PollEvented};
use tokio_file_unix::File;

pub mod config;
pub mod rep;
pub mod req;
pub mod zpub;
pub mod sub;
pub mod push;
pub mod pull;
pub mod xpub;
pub mod xsub;
pub mod pair;

pub use self::rep::{Rep, RepControlled};
pub use self::req::Req;
pub use self::zpub::Pub;
pub use self::sub::{Sub, SubControlled};
pub use self::push::Push;
pub use self::pull::{Pull, PullControlled};
pub use self::xpub::{Xpub, XpubControlled};
pub use self::xsub::{Xsub, XsubControlled};
pub use self::pair::{Pair, PairControlled};

use self::config::SockConfigStart;
use async::{ControlledStream, ControlHandler, Multipart, MultipartRequest, MultipartResponse,
            MultipartSink, MultipartStream};
use error::Error;
use ZmqFile;

pub trait AsSocket {
    fn socket(&self) -> &Socket;

    fn into_socket(self) -> Socket;
}

pub trait ControlledStreamSocket<H>
where
    H: ControlHandler,
{
    fn socket(&self) -> &ControlledSocket;

    fn recv(&self) -> MultipartResponse {
        self.socket().recv()
    }

    fn stream(&self, handler: H) -> ControlledStream<H> {
        self.socket().stream(handler)
    }
}

pub trait StreamSocket: AsSocket {
    fn recv(&self) -> MultipartResponse {
        self.socket().recv()
    }

    fn stream(&self) -> MultipartStream {
        self.socket().stream()
    }
}

pub trait SinkSocket: AsSocket {
    fn send(&self, multipart: Multipart) -> MultipartRequest {
        self.socket().send(multipart)
    }

    fn sink<E>(&self) -> MultipartSink<E>
    where
        E: From<Error>,
    {
        self.socket().sink()
    }
}

pub trait FutureSocket: AsSocket {
    fn send(&self, multipart: Multipart) -> MultipartRequest {
        self.socket().send(multipart)
    }

    fn recv(&self) -> MultipartResponse {
        self.socket().recv()
    }
}

pub struct Socket {
    // Reads and Writes data
    sock: Rc<zmq::Socket>,
    // So we can hand out files to streams and sinks
    file: Rc<PollEvented<File<ZmqFile>>>,
}

impl Socket {
    pub fn new(ctx: Rc<zmq::Context>, handle: Handle) -> SockConfigStart {
        SockConfigStart::new(ctx, handle)
    }

    pub fn from_sock_and_file(sock: Rc<zmq::Socket>, file: Rc<PollEvented<File<ZmqFile>>>) -> Self {
        Socket { sock, file }
    }

    pub fn controlled<S>(self, control: S) -> ControlledSocket
    where
        S: StreamSocket,
    {
        ControlledSocket {
            stream_sock: self,
            control_sock: control.into_socket(),
        }
    }

    pub fn sink<E>(&self) -> MultipartSink<E>
    where
        E: From<Error>,
    {
        MultipartSink::new(Rc::clone(&self.sock), Rc::clone(&self.file))
    }

    pub fn stream(&self) -> MultipartStream {
        MultipartStream::new(Rc::clone(&self.sock), Rc::clone(&self.file))
    }

    pub fn send(&self, multipart: Multipart) -> MultipartRequest {
        MultipartRequest::new(Rc::clone(&self.sock), Rc::clone(&self.file), multipart)
    }

    pub fn recv(&self) -> MultipartResponse {
        MultipartResponse::new(Rc::clone(&self.sock), Rc::clone(&self.file))
    }
}

pub struct ControlledSocket {
    stream_sock: Socket,
    control_sock: Socket,
}

impl ControlledSocket {
    pub fn sink<E>(&self) -> MultipartSink<E>
    where
        E: From<Error>,
    {
        self.stream_sock.sink()
    }

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

    pub fn send(&self, multipart: Multipart) -> MultipartRequest {
        self.stream_sock.send(multipart)
    }

    pub fn recv(&self) -> MultipartResponse {
        self.stream_sock.recv()
    }
}
