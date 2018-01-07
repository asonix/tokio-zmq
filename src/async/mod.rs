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

//! This module contains the code that makes Tokio ZMQ Asynchronous. There's the `future` module,
//! which defines Request and Response futures for ZeroMQ Sockets, the `stream` module, which
//! defines receiving data from a socket as an asychronous stream, and the `sink` module, which
//! defines sending data to a socket as an asychronous sink.

pub mod future;
pub mod sink;
pub mod stream;

pub use self::future::{MultipartRequest, MultipartResponse};
pub use self::sink::MultipartSink;
pub use self::stream::{ControlledStream, MultipartStream};

/// This type is used to determine what flags should be used when sending messages. If a message is
/// the last in it's `Multipart`, it should not have the SNDMORE flag set.
#[derive(PartialEq)]
pub enum MsgPlace {
    Nth,
    Last,
}
