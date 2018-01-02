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

use std::collections::VecDeque;

use zmq;

pub use self::future::{MultipartRequest, MultipartResponse};
pub use self::sink::MultipartSink;
pub use self::stream::{ControlledStream, ControlHandler, MultipartStream};

/// This type is used for receiving and sending messages in Multipart groups. An application could
/// make using this easier by implementing traits as follows:
///
/// ```rust
/// #![feature(try_from)]
///
/// extern crate zmq;
/// extern crate tokio_zmq;
///
/// use std::convert::{TryFrom, TryInto};
/// use std::collections::VecDeque;
///
/// use tokio_zmq::async::Multipart;
///
/// #[derive(Debug)]
/// enum Error {
///     NotEnoughMessages,
///     TooManyMessages,
/// }
///
/// struct Envelope {
///     filter: zmq::Message,
///     address: zmq::Message,
///     body: zmq::Message,
/// }
///
/// impl TryFrom<Multipart> for Envelope {
///     type Error = Error;
///
///     fn try_from(mut multipart: Multipart) -> Result<Self, Self::Error> {
///         let filter = multipart.pop_front().ok_or(Error::NotEnoughMessages)?;
///         let address = multipart.pop_front().ok_or(Error::NotEnoughMessages)?;
///         let body = multipart.pop_front().ok_or(Error::NotEnoughMessages)?;
///
///         if !multipart.is_empty() {
///             return Err(Error::TooManyMessages);
///         }
///
///         Ok(Envelope {
///             filter,
///             address,
///             body,
///         })
///     }
/// }
///
/// impl From<Envelope> for Multipart {
///     fn from(envelope: Envelope) -> Self {
///         let mut multipart = VecDeque::new();
///
///         multipart.push_back(envelope.filter);
///         multipart.push_back(envelope.address);
///         multipart.push_back(envelope.body);
///
///         multipart
///     }
/// }
///
/// fn main() {
///     let mut multipart: Multipart = VecDeque::new();
///     multipart.push_back(zmq::Message::from_slice(b"FILTER: asdf").unwrap());
///     multipart.push_back(zmq::Message::from_slice(b"some.address").unwrap());
///     multipart.push_back(zmq::Message::from_slice(b"Some content").unwrap());
///     let envelope: Envelope = multipart.try_into().unwrap();
///
///     let multipart2: Multipart = envelope.into();
/// }
/// ```
pub type Multipart = VecDeque<zmq::Message>;

/// This type is used to determine what flags should be used when sending messages. If a message is
/// the last in it's `Multipart`, it should not have the SNDMORE flag set.
#[derive(PartialEq)]
pub enum MsgPlace {
    Nth,
    Last,
}
