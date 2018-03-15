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

//! This module contains `SocketBuilder` and related types.

use std::rc::Rc;

use zmq;
use tokio::reactor::PollEvented2;
use tokio_file_unix::File;

use socket::Socket;
use error::Error;
use file::ZmqFile;

fn bind_all(sock: zmq::Socket, binds: &[&str]) -> zmq::Result<zmq::Socket> {
    for bind in binds {
        sock.bind(bind)?;
    }
    Ok(sock)
}

fn connect_all(sock: zmq::Socket, connects: &[&str]) -> zmq::Result<zmq::Socket> {
    for connect in connects {
        sock.connect(connect)?;
    }
    Ok(sock)
}

/// The root struct for a Socket builder
///
/// This struct contains a context and an identity.
pub struct SocketBuilder<'a> {
    ctx: Rc<zmq::Context>,
    identity: Option<&'a [u8]>,
}

impl<'a> SocketBuilder<'a> {
    /// Create a new Socket builder
    ///
    /// All sockets that are created through the Tokio ZMQ library will use this as the base for
    /// their socket builder (except PAIR sockets).
    pub fn new(ctx: Rc<zmq::Context>) -> Self {
        SocketBuilder {
            ctx: ctx,
            identity: None,
        }
    }

    /// Give the socket a custom identity
    pub fn identity(self, identity: &'a [u8]) -> Self {
        SocketBuilder {
            ctx: self.ctx,
            identity: Some(identity),
        }
    }

    /// Bind the socket to an address
    ///
    /// Since this is just part of the builder, and the socket doesn't exist yet, we store the
    /// address for later retrieval.
    pub fn bind(self, addr: &'a str) -> SockConfig<'a> {
        let mut bind = Vec::new();
        bind.push(addr);

        SockConfig {
            ctx: self.ctx,
            bind: bind,
            connect: Vec::new(),
            identity: self.identity,
        }
    }

    /// Connect the socket to an address
    ///
    /// Since this is just part of the builder, and the socket doesn't exist yet, we store the
    /// address for later retrieval.
    pub fn connect(self, addr: &'a str) -> SockConfig<'a> {
        let mut connect = Vec::new();
        connect.push(addr);

        SockConfig {
            ctx: self.ctx,
            bind: Vec::new(),
            connect: connect,
            identity: self.identity,
        }
    }

    /// Bind or Connect the socket to an address
    ///
    /// This method indicates that the resulting socket will be a PAIR socket.
    pub fn pair(self, addr: &'a str, bind: bool) -> PairConfig<'a> {
        PairConfig {
            ctx: self.ctx,
            addr: addr,
            bind: bind,
            identity: self.identity,
        }
    }
}

/// The final builder step for some socket types
///
/// This contains all the information required to contstruct a valid socket, except in the case of
/// SUB, which needs an additional `filter` parameter.
pub struct SockConfig<'a> {
    pub ctx: Rc<zmq::Context>,
    pub bind: Vec<&'a str>,
    pub connect: Vec<&'a str>,
    pub identity: Option<&'a [u8]>,
}

impl<'a> SockConfig<'a> {
    /// Bind the `SockConfig` to an address, returning a `SockConfig`
    ///
    /// This allows for a single socket to be bound to multiple addresses.
    pub fn bind(mut self, addr: &'a str) -> Self {
        self.bind.push(addr);
        self
    }

    /// Connect the `SockConfig` to an address, returning a `SockConfig`
    ///
    /// This allows for a single socket to be connected to multiple addresses.
    pub fn connect(mut self, addr: &'a str) -> Self {
        self.connect.push(addr);
        self
    }

    /// Finalize the `SockConfig` into a `Socket` if the creation is successful, or into an Error
    /// if something went wrong.
    ///
    /// Since we can't dynamically create different wrapper types (Rep, Req, Pub, etc.), we just
    /// create the inner Socket, and expect this function to be called in a few contexts
    ///
    ///  - The caller is a wrapper type.
    ///  - The caller knows what they're doing.
    ///
    /// For convenience, `TryFrom<SockConfig>` is implemented for all valid wrapper types.
    pub fn build(self, kind: zmq::SocketType) -> Result<Socket, Error> {
        let SockConfig {
            ctx,
            bind,
            connect,
            identity,
        } = self;

        let sock = ctx.socket(kind)?;
        if let Some(identity) = identity {
            sock.set_identity(identity)?;
        }
        let sock = bind_all(sock, &bind)?;
        let sock = connect_all(sock, &connect)?;

        let fd = sock.get_fd()?;

        let file = PollEvented2::new(File::new_nb(ZmqFile::from_raw_fd(fd))?);

        Ok(Socket::from_sock_and_file(sock, file))
    }

    /// Continue the building process into a SubConfig, for the SUB socket type which requires
    /// setting a subscription filter.
    pub fn filter(self, pattern: &'a [u8]) -> SubConfig<'a> {
        SubConfig {
            ctx: self.ctx,
            bind: self.bind,
            connect: self.connect,
            identity: self.identity,
            filter: pattern,
        }
    }
}

/// The final builder step for the Sub socket type.
///
/// This contains all the information required to contstruct a valid SUB socket
pub struct SubConfig<'a> {
    pub ctx: Rc<zmq::Context>,
    pub bind: Vec<&'a str>,
    pub connect: Vec<&'a str>,
    pub filter: &'a [u8],
    pub identity: Option<&'a [u8]>,
}

impl<'a> SubConfig<'a> {
    /// Finalize the `SubConfig` into a `Socket` if the creation is successful, or into an Error
    /// if something went wrong.
    ///
    /// We do know the type of socket this should end up being, but to maintain convention with
    /// `SockConfig`'s build, this method creates a 'raw' `Socket` type. This function should only
    /// be called in a few contexts:
    ///
    ///  - The caller is the Sub wrapper type.
    ///  - The caller knows what they're doing.
    ///
    /// For convenience, `TryFrom<SockConfig>` is implemented for all valid wrapper types.
    pub fn build(self, _: zmq::SocketType) -> Result<Socket, Error> {
        let SubConfig {
            ctx,
            bind,
            connect,
            filter,
            identity,
        } = self;

        let sock = ctx.socket(zmq::SUB)?;
        if let Some(identity) = identity {
            sock.set_identity(identity)?;
        }
        let sock = bind_all(sock, &bind)?;
        let sock = connect_all(sock, &connect)?;
        sock.set_subscribe(filter)?;

        let fd = sock.get_fd()?;

        let file = PollEvented2::new(File::new_nb(ZmqFile::from_raw_fd(fd))?);

        Ok(Socket::from_sock_and_file(sock, file))
    }
}

/// The final builder step for the Pair socket type.
///
/// This contains all the information required to contstruct a valid PAIR socket
pub struct PairConfig<'a> {
    ctx: Rc<zmq::Context>,
    addr: &'a str,
    bind: bool,
    identity: Option<&'a [u8]>,
}

impl<'a> PairConfig<'a> {
    /// Construct a raw `Socket` type from the given `PairConfig`
    ///
    /// This build takes the same arguments as the `SockConfig`'s build method for convenience, but
    /// this should not be called with `zmq::SocketType`s other than `zmq::PAIR`. The `Pair`
    /// wrapper uses this builder, so it is better to use the Pair wrapper than directly building a
    /// PAIR socket.
    pub fn build(self, _: zmq::SocketType) -> Result<Socket, Error> {
        let PairConfig {
            ctx,
            addr,
            bind,
            identity,
        } = self;

        let sock = ctx.socket(zmq::PAIR)?;
        if let Some(identity) = identity {
            sock.set_identity(identity)?;
        }
        if bind {
            sock.bind(addr)?;
        } else {
            sock.connect(addr)?;
        }

        let fd = sock.get_fd()?;

        let file = PollEvented2::new(File::new_nb(ZmqFile::from_raw_fd(fd))?);

        Ok(Socket::from_sock_and_file(sock, file))
    }
}
