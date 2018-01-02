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

use std::convert::TryFrom;

use zmq;

use socket::config::SockConfig;
use socket::{AsSocket, ControlledSocket, ControlledSinkSocket, ControlledStreamSocket,
             ControlHandler, SinkSocket, Socket, StreamSocket};
use error::Error;

pub struct Xpub {
    inner: Socket,
}

impl Xpub {
    pub fn controlled<S>(self, control: S) -> XpubControlled
    where
        S: StreamSocket,
    {
        XpubControlled { inner: self.inner.controlled(control) }
    }
}

impl AsSocket for Xpub {
    fn socket(&self) -> &Socket {
        &self.inner
    }

    fn into_socket(self) -> Socket {
        self.inner
    }
}

impl StreamSocket for Xpub {}
impl SinkSocket for Xpub {}

impl<'a> TryFrom<SockConfig<'a>> for Xpub {
    type Error = Error;

    fn try_from(conf: SockConfig<'a>) -> Result<Self, Self::Error> {
        Ok(Xpub { inner: conf.build(zmq::XPUB)? })
    }
}

pub struct XpubControlled {
    inner: ControlledSocket,
}

impl<H> ControlledStreamSocket<H> for XpubControlled
where
    H: ControlHandler,
{
    fn socket(&self) -> &ControlledSocket {
        &self.inner
    }
}

impl<H> ControlledSinkSocket<H> for XpubControlled
where
    H: ControlHandler,
{
}
