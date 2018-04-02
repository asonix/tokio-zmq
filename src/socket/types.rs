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

//! This module defines all the socket wrapper types that can be used with Tokio.

use std::convert::TryFrom;

use tokio_reactor::PollEvented;
use tokio_file_unix::File;
use zmq;

use error::Error;
use file::ZmqFile;
use socket::config::{PairConfig, SockConfig, SubConfig};
use socket::Socket;

/* -------------------------------------------------------------------------- */

/// The DEALER `SocketType` wrapper type.
///
/// Dealer implements `StreamSocket` and `SinkSocket`, and has an associated controlled variant.
#[derive(SocketWrapper)]
#[stream]
#[sink]
pub struct Dealer {
    inner: Socket,
}

/* -------------------------------------------------------------------------- */

/// The PAIR `SocketType` wrapper type.
///
/// Pair implements `StreamSocket` and `SinkSocket`, and has an associated controlled variant.
#[derive(SocketWrapper)]
#[stream]
#[sink]
#[try_from = "PairConfig"]
pub struct Pair {
    inner: Socket,
}

/* -------------------------------------------------------------------------- */

/// The PUB `SocketType` wrapper type
///
/// Pub implements `SinkSocket`.
#[derive(SocketWrapper)]
#[sink]
pub struct Pub {
    inner: Socket,
}

/* -------------------------------------------------------------------------- */

/// The PULL `SocketType` wrapper type
///
/// Pull implements `StreamSocket`, and has an associated controlled variant.
#[derive(SocketWrapper)]
#[stream]
pub struct Pull {
    inner: Socket,
}

/* -------------------------------------------------------------------------- */

/// The PUSH `SocketType` wrapper type
///
/// Push implements `SinkSocket`.
#[derive(SocketWrapper)]
#[sink]
pub struct Push {
    inner: Socket,
}

/* -------------------------------------------------------------------------- */

/// The REP `SocketType` wrapper type
///
/// Rep implements `StreamSocket` and `SinkSocket`, and has an associated controlled variant.
#[derive(SocketWrapper)]
#[stream]
#[sink]
pub struct Rep {
    inner: Socket,
}

/* -------------------------------------------------------------------------- */

/// The REQ `SocketType` wrapper type
///
/// Req implements `StreamSocket` and `SinkSocket`, and has an associated controlled variant.
#[derive(SocketWrapper)]
#[stream]
#[sink]
pub struct Req {
    inner: Socket,
}

/* -------------------------------------------------------------------------- */

/// The ROUTER `SocketType` wrapper type
///
/// Router implements `StreamSocket` and `SinkSocket`, and has an associated controlled variant.
#[derive(SocketWrapper)]
#[stream]
#[sink]
pub struct Router {
    inner: Socket,
}

/* -------------------------------------------------------------------------- */

/// The SUB `SocketType` wrapper type
///
/// Sub implements `StreamSocket`, and has an associated controlled variant.
#[derive(SocketWrapper)]
#[stream]
#[try_from = "SubConfig"]
pub struct Sub {
    inner: Socket,
}

/* -------------------------------------------------------------------------- */

/// The XPUB `SocketType` wrapper type
///
/// Xpub implements `StreamSocket` and `SinkSocket`, and has an associated controlled variant.
#[derive(SocketWrapper)]
#[stream]
#[sink]
pub struct Xpub {
    inner: Socket,
}

/* -------------------------------------------------------------------------- */

/// The XSUB `SocketType` wrapper type
///
/// Xsub implements `StreamSocket` and `SinkSocket`, and has an associated controlled variant.
#[derive(SocketWrapper)]
#[stream]
#[sink]
pub struct Xsub {
    inner: Socket,
}
