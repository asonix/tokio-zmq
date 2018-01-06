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

//! This module contains definitions for the `ZmqFile` type, a small wrapper around a `RawFd` so
//! `tokio_file_unix` can interact with it.

use std::os::unix::io::{AsRawFd, RawFd};

/// Create a simple wraper struct to hand to `tokio_file_unix`'s File `new_nb` constructor
pub struct ZmqFile {
    fd: RawFd,
}

impl ZmqFile {
    /// Create a ZmqFile from a file descriptor
    pub fn from_raw_fd(fd: RawFd) -> Self {
        ZmqFile { fd }
    }
}

impl AsRawFd for ZmqFile {
    /// ZmqFile must implement `AsRawFd` to be compatable with `tokio_file_unix`
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}
