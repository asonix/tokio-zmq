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

use socket::Socket;
use error::Error;
use ZmqFile;

pub fn bind_all(sock: zmq::Socket, binds: &[String]) -> zmq::Result<zmq::Socket> {
    for bind in binds {
        sock.bind(bind)?;
    }
    Ok(sock)
}

pub fn connect_all(sock: zmq::Socket, connects: &[String]) -> zmq::Result<zmq::Socket> {
    for connect in connects {
        sock.connect(connect)?;
    }
    Ok(sock)
}

pub struct SockConfigStart {
    ctx: Rc<zmq::Context>,
    handle: Handle,
}

impl SockConfigStart {
    pub fn new(ctx: Rc<zmq::Context>, handle: Handle) -> Self {
        SockConfigStart { ctx, handle }
    }

    pub fn bind(self, addr: String) -> SockConfig {
        let mut bind = Vec::new();
        bind.push(addr);

        SockConfig {
            ctx: self.ctx,
            handle: self.handle,
            bind: bind,
            connect: Vec::new(),
        }
    }

    pub fn connect(self, addr: String) -> SockConfig {
        let mut connect = Vec::new();
        connect.push(addr);

        SockConfig {
            ctx: self.ctx,
            handle: self.handle,
            bind: Vec::new(),
            connect: connect,
        }
    }
}

pub struct SockConfig {
    pub ctx: Rc<zmq::Context>,
    pub handle: Handle,
    pub bind: Vec<String>,
    pub connect: Vec<String>,
}

impl SockConfig {
    pub fn new(ctx: Rc<zmq::Context>, handle: Handle) -> SockConfigStart {
        SockConfigStart::new(ctx, handle)
    }

    pub fn bind(mut self, addr: String) -> Self {
        self.bind.push(addr);
        self
    }

    pub fn connect(mut self, addr: String) -> Self {
        self.connect.push(addr);
        self
    }

    pub fn build(self, kind: zmq::SocketType) -> Result<Socket, Error> {
        let SockConfig {
            ctx,
            handle,
            bind,
            connect,
        } = self;

        let sock = ctx.socket(kind)?;
        let sock = bind_all(sock, &bind)?;
        let sock = connect_all(sock, &connect)?;

        let fd = sock.get_fd()?;

        let sock = Rc::new(sock);

        let file = Rc::new(PollEvented::new(
            File::new_nb(ZmqFile::from_raw_fd(fd))?,
            &handle,
        )?);

        Ok(Socket::from_sock_and_file(sock, file))
    }

    pub fn filter(self, pattern: Vec<u8>) -> SubConfig {
        SubConfig {
            ctx: self.ctx,
            handle: self.handle,
            bind: self.bind,
            connect: self.connect,
            filter: pattern,
        }
    }
}

pub struct SubConfig {
    pub ctx: Rc<zmq::Context>,
    pub handle: Handle,
    pub bind: Vec<String>,
    pub connect: Vec<String>,
    pub filter: Vec<u8>,
}

impl SubConfig {
    pub fn new(ctx: Rc<zmq::Context>, handle: Handle) -> SockConfigStart {
        SockConfigStart::new(ctx, handle)
    }

    pub fn build(self, kind: zmq::SocketType) -> Result<Socket, Error> {
        let SubConfig {
            ctx,
            handle,
            bind,
            connect,
            filter,
        } = self;

        let sock = ctx.socket(kind)?;
        let sock = bind_all(sock, &bind)?;
        let sock = connect_all(sock, &connect)?;
        sock.set_subscribe(&filter)?;

        let fd = sock.get_fd()?;

        let sock = Rc::new(sock);

        let file = Rc::new(PollEvented::new(
            File::new_nb(ZmqFile::from_raw_fd(fd))?,
            &handle,
        )?);

        Ok(Socket::from_sock_and_file(sock, file))
    }
}

/*
pub struct PairConfigStart {
    ctx: Rc<zmq::Context>,
    handle: Handle,
}

impl PairConfigStart {
    pub fn new(ctx: Rc<zmq::Context>, handle: Handle) -> Self {
        PairConfigStart { ctx, handle }
    }

    pub fn bind(self, addr: String) -> PairConfig {
        PairConfig {
            ctx: self.ctx,
            handle: self.handle,
            addr: addr,
            bind: true,
        }
    }

    pub fn connect(self, addr: String) -> PairConfig {
        PairConfig {
            ctx: self.ctx,
            handle: self.handle,
            addr: addr,
            bind: false,
        }
    }
}

pub struct PairConfig {
    ctx: Rc<zmq::Context>,
    handle: Handle,
    addr: String,
    bind: bool,
}

impl PairConfig {
    pub fn new(ctx: Rc<zmq::Context>, handle: Handle) -> PairConfigStart {
        PairConfigStart::new(ctx, handle)
    }

    pub fn build<S>(self) -> Result<S, S::Error>
    where
        S: TryFrom<PairConfig>,
    {
        self.try_into()
    }
}
*/
