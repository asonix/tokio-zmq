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

//! This module contains the Multipart type, which is a wrapper around a VecDeque. The Multipart
//! type implements `From<zmq::Message>` for easy creation.

use std::collections::VecDeque;
use std::collections::vec_deque::{IntoIter, Iter, IterMut};

use zmq;

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
///
/// use tokio_zmq::Multipart;
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
///         let mut multipart = Multipart::new();
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
///     let mut multipart: Multipart = Multipart::new();
///     multipart.push_back(zmq::Message::from_slice(b"FILTER: asdf").unwrap());
///     multipart.push_back(zmq::Message::from_slice(b"some.address").unwrap());
///     multipart.push_back(zmq::Message::from_slice(b"Some content").unwrap());
///     let envelope: Envelope = multipart.try_into().unwrap();
///
///     let multipart2: Multipart = envelope.into();
/// }
/// ```
#[derive(Debug)]
pub struct Multipart {
    inner: VecDeque<zmq::Message>,
}

impl Multipart {
    pub fn new() -> Self {
        Multipart::default()
    }

    pub fn into_inner(self) -> VecDeque<zmq::Message> {
        self.inner
    }

    pub fn get(&self, index: usize) -> Option<&zmq::Message> {
        self.inner.get(index)
    }

    pub fn pop_front(&mut self) -> Option<zmq::Message> {
        self.inner.pop_front()
    }

    pub fn pop_back(&mut self) -> Option<zmq::Message> {
        self.inner.pop_back()
    }

    pub fn push_front(&mut self, msg: zmq::Message) {
        self.inner.push_front(msg)
    }

    pub fn push_back(&mut self, msg: zmq::Message) {
        self.inner.push_back(msg)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> Iter<zmq::Message> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<zmq::Message> {
        self.inner.iter_mut()
    }
}

impl Default for Multipart {
    fn default() -> Self {
        Multipart {
            inner: VecDeque::new(),
        }
    }
}

impl From<zmq::Message> for Multipart {
    fn from(msg: zmq::Message) -> Self {
        let mut multipart = Multipart::new();
        multipart.push_back(msg);
        multipart
    }
}

impl From<Vec<zmq::Message>> for Multipart {
    fn from(v: Vec<zmq::Message>) -> Self {
        Multipart { inner: v.into() }
    }
}

impl<'a> IntoIterator for &'a Multipart {
    type Item = &'a zmq::Message;
    type IntoIter = Iter<'a, zmq::Message>;

    fn into_iter(self) -> Iter<'a, zmq::Message> {
        self.iter()
    }
}

impl IntoIterator for Multipart {
    type Item = zmq::Message;
    type IntoIter = IntoIter<zmq::Message>;

    fn into_iter(self) -> IntoIter<zmq::Message> {
        self.inner.into_iter()
    }
}

impl<'a> IntoIterator for &'a mut Multipart {
    type Item = &'a mut zmq::Message;
    type IntoIter = IterMut<'a, zmq::Message>;

    fn into_iter(self) -> IterMut<'a, zmq::Message> {
        self.iter_mut()
    }
}
