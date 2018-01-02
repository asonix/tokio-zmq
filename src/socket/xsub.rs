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

pub struct Xsub {
    inner: Socket,
}

impl Xsub {
    pub fn control<S>(self, control: S) -> XsubControlled
    where
        S: StreamSocket,
    {
        XsubControlled { inner: self.inner.controlled(control) }
    }
}

impl AsSocket for Xsub {
    fn socket(&self) -> &Socket {
        &self.inner
    }

    fn into_socket(self) -> Socket {
        self.inner
    }
}

impl StreamSocket for Xsub {}
impl SinkSocket for Xsub {}

impl<'a> TryFrom<SockConfig<'a>> for Xsub {
    type Error = Error;

    fn try_from(conf: SockConfig<'a>) -> Result<Self, Self::Error> {
        Ok(Xsub { inner: conf.build(zmq::XSUB)? })
    }
}

pub struct XsubControlled {
    inner: ControlledSocket,
}

impl<H> ControlledStreamSocket<H> for XsubControlled
where
    H: ControlHandler,
{
    fn socket(&self) -> &ControlledSocket {
        &self.inner
    }
}

impl<H> ControlledSinkSocket<H> for XsubControlled
where
    H: ControlHandler,
{
}
