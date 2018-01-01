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

#![feature(conservative_impl_trait)]
#![feature(try_from)]

extern crate zmq;
extern crate futures;
extern crate tokio_core;
extern crate tokio_file_unix;
#[macro_use]
extern crate log;

pub mod async;
pub mod error;
pub mod socket;

pub use async::ControlHandler;
pub use self::error::Error;
pub use socket::{AsSocket, FutureSocket, SinkSocket, Socket, StreamSocket};
pub use socket::{Rep, Req, Pub, Sub, Push, Pull, Xpub, Xsub, Pair};
pub use socket::{RepControlled, SubControlled, PullControlled, XpubControlled, XsubControlled,
                 PairControlled};
pub use socket::ControlledStreamSocket;

use std::os::unix::io::{AsRawFd, RawFd};

pub struct ZmqFile {
    fd: RawFd,
}

impl ZmqFile {
    fn from_raw_fd(fd: RawFd) -> Self {
        ZmqFile { fd }
    }
}

impl AsRawFd for ZmqFile {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}
