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

use zmq;

use socket::config::{PairConfig, SockConfig, SubConfig};
use socket::{ControlledSocket, Socket};
use error::Error;

/* -------------------------------------------------------------------------- */

/// The DEALER `SocketType` wrapper type.
///
/// Dealer implements `StreamSocket` and `SinkSocket`, and has an associated controlled variant.
#[derive(Clone, SocketWrapper)]
#[stream]
#[sink]
#[controlled = "DealerControlled"]
pub struct Dealer {
    inner: Socket,
}

/// The controlled variant of Dealer
#[derive(Clone, ControlledSocketWrapper)]
#[sink]
pub struct DealerControlled {
    inner: ControlledSocket,
}

/* -------------------------------------------------------------------------- */

/// The PAIR `SocketType` wrapper type.
///
/// Pair implements `StreamSocket` and `SinkSocket`, and has an associated controlled variant.
#[derive(Clone, SocketWrapper)]
#[stream]
#[sink]
#[controlled = "PairControlled"]
#[try_from = "PairConfig"]
pub struct Pair {
    inner: Socket,
}

/// The controlled variant of Pair
#[derive(Clone, ControlledSocketWrapper)]
#[sink]
pub struct PairControlled {
    inner: ControlledSocket,
}

/* -------------------------------------------------------------------------- */

/// The PUB `SocketType` wrapper type
///
/// Pub implements `SinkSocket`.
#[derive(Clone, SocketWrapper)]
#[sink]
pub struct Pub {
    inner: Socket,
}

/* -------------------------------------------------------------------------- */

/// The PULL `SocketType` wrapper type
///
/// Pull implements `StreamSocket`, and has an associated controlled variant.
#[derive(Clone, SocketWrapper)]
#[stream]
#[controlled = "PullControlled"]
pub struct Pull {
    inner: Socket,
}

/// The controlled variant of Pull
#[derive(Clone, ControlledSocketWrapper)]
pub struct PullControlled {
    inner: ControlledSocket,
}

/* -------------------------------------------------------------------------- */

/// The PUSH `SocketType` wrapper type
///
/// Push implements `SinkSocket`.
#[derive(Clone, SocketWrapper)]
#[sink]
pub struct Push {
    inner: Socket,
}

/* -------------------------------------------------------------------------- */

/// The REP `SocketType` wrapper type
///
/// Rep implements `StreamSocket` and `SinkSocket`, and has an associated controlled variant.
#[derive(Clone, SocketWrapper)]
#[stream]
#[sink]
#[controlled = "RepControlled"]
pub struct Rep {
    inner: Socket,
}

/// The controlled variant of Rep
#[derive(Clone, ControlledSocketWrapper)]
#[sink]
pub struct RepControlled {
    inner: ControlledSocket,
}

/* -------------------------------------------------------------------------- */

/// The REQ `SocketType` wrapper type
///
/// Req implements `FutureSocket`.
#[derive(Clone, SocketWrapper)]
#[future]
pub struct Req {
    inner: Socket,
}

/* -------------------------------------------------------------------------- */

/// The ROUTER `SocketType` wrapper type
///
/// Router implements `StreamSocket` and `SinkSocket`, and has an associated controlled variant.
#[derive(Clone, SocketWrapper)]
#[stream]
#[sink]
#[controlled = "RouterControlled"]
pub struct Router {
    inner: Socket,
}

/// The controlled variant of Router
#[derive(Clone, ControlledSocketWrapper)]
#[sink]
pub struct RouterControlled {
    inner: ControlledSocket,
}

/* -------------------------------------------------------------------------- */

/// The SUB `SocketType` wrapper type
///
/// Sub implements `StreamSocket`, and has an associated controlled variant.
#[derive(Clone, SocketWrapper)]
#[stream]
#[controlled = "SubControlled"]
#[try_from = "SubConfig"]
pub struct Sub {
    inner: Socket,
}

/// The controlled variant of Sub.
#[derive(Clone, ControlledSocketWrapper)]
pub struct SubControlled {
    inner: ControlledSocket,
}

/* -------------------------------------------------------------------------- */

/// The XPUB `SocketType` wrapper type
///
/// Xpub implements `StreamSocket` and `SinkSocket`, and has an associated controlled variant.
#[derive(Clone, SocketWrapper)]
#[stream]
#[sink]
#[controlled = "XpubControlled"]
pub struct Xpub {
    inner: Socket,
}

/// The controlled variant of Xpub
#[derive(Clone, ControlledSocketWrapper)]
#[sink]
pub struct XpubControlled {
    inner: ControlledSocket,
}

/* -------------------------------------------------------------------------- */

/// The XSUB `SocketType` wrapper type
///
/// Xsub implements `StreamSocket` and `SinkSocket`, and has an associated controlled variant.
#[derive(Clone, SocketWrapper)]
#[stream]
#[sink]
#[controlled = "XsubControlled"]
pub struct Xsub {
    inner: Socket,
}

/// The controlled variant of Xsub
#[derive(Clone, ControlledSocketWrapper)]
#[sink]
pub struct XsubControlled {
    inner: ControlledSocket,
}
