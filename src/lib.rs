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

//! Tokio ZMQ, bringing Zero MQ to the Tokio event loop
//!
//! This crate provides Streams, Sinks, and Futures for Zero MQ Sockets, which deal in structures
//! caled Multiparts. Currently, a Multipart is a simple VecDeque<zmq::Message>, but possibly in
//! the future this can be represented as a struct, or VecDeque<S: zmq::Sendable> with the zmq 0.9
//! release.
//!
//! # Creating a socket
//!
//! To get a new socket, you must invoke the Socket builder. The Socket Builder can output a
//! 'raw' Socket, or any specific kind of socket, such as Rep, Req, etc. The result of the builder
//! can be any compatable kind of socket, so specifiying a type is important.
//!
//! Once you have a socket, if it implements `StreamSocket`, you can use the socket's `.stream()`, if
//! it implements `SinkSocket`, you can use the socket's `.sink()`, and if it implements
//! `FutureSocket`, you can use the `send` and `recv` methods.
//!
//! Without further ado, creating and using a socket:
//!
//! ```rust
//! #![feature(try_from)]
//!
//! extern crate zmq;
//! extern crate futures;
//! extern crate tokio_core;
//! extern crate tokio_zmq;
//!
//! use std::convert::TryInto;
//! use std::rc::Rc;
//!
//! use futures::Stream;
//! use tokio_core::reactor::Core;
//! use tokio_zmq::prelude::*;
//! use tokio_zmq::{Socket, Pub, Sub, Error};
//!
//! fn run() -> Result<(), Error> {
//!     // Create a new Event Loop. Typically this will happen somewhere near the start of your
//!     // application.
//!     let mut core = Core::new()?;
//!
//!     // Create a new ZeroMQ Context. This context will be used to create all the sockets.
//!     let context = Rc::new(zmq::Context::new());
//!
//!     // Create our two sockets using the Socket builder pattern.
//!     // Note that the variable is named zpub, since pub is a keyword
//!     let zpub: Pub = Socket::new(Rc::clone(&context), core.handle())
//!         .bind("tcp://*:5561")
//!         .try_into()?;
//!
//!     let sub: Sub = Socket::new(context, core.handle())
//!         .bind("tcp://*:5562")
//!         .filter(b"")
//!         .try_into()?;
//!
//!     // Create our simple server. This forwards messages from the Subscriber socket to the
//!     // Publisher socket, and prints them as they go by.
//!     let runner = sub.stream()
//!         .map(|multipart| {
//!             for msg in &multipart {
//!                 if let Some(msg) = msg.as_str() {
//!                     println!("Forwarding: {}", msg);
//!                 }
//!             }
//!             multipart
//!         })
//!         .forward(zpub.sink::<Error>());
//!
//!     // To avoid an infinte doctest, the actual core.run is commented out.
//!     // core.run(runner)?;
//!     # let _ = runner;
//!     # Ok(())
//! }
//!
//! # fn main() {
//! #     run().unwrap();
//! # }
//! ```

extern crate zmq;
extern crate futures;
extern crate tokio_core;
extern crate tokio_file_unix;
#[macro_use]
extern crate log;

mod error;
pub mod async;
pub mod socket;
pub mod file;
pub mod prelude;

pub use self::error::Error;
pub use self::socket::Socket;
pub use self::socket::{Dealer, Rep, Req, Router, Pub, Sub, Push, Pull, Xpub, Xsub, Pair};
