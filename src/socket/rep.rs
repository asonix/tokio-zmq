/*
 * This file is part of ZeroMQ Futures.
 *
 * Copyright © 2017 Riley Trautman
 *
 * ZeroMQ Futures is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * ZeroMQ Futures is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with ZeroMQ Futures.  If not, see <http://www.gnu.org/licenses/>.
 */

use std::convert::TryFrom;

use zmq;

use socket::config::SockConfig;
use socket::{AsSocket, ControlledSocket, ControlledStreamSocket, ControlHandler, SinkSocket,
             Socket, StreamSocket};
use error::Error;

pub struct Rep {
    inner: Socket,
}

impl Rep {
    pub fn controlled<S>(self, control: S) -> RepControlled
    where
        S: StreamSocket,
    {
        RepControlled { inner: self.inner.controlled(control) }
    }
}

impl AsSocket for Rep {
    fn socket(&self) -> &Socket {
        &self.inner
    }

    fn into_socket(self) -> Socket {
        self.inner
    }
}

impl StreamSocket for Rep {}
impl SinkSocket for Rep {}

impl TryFrom<SockConfig> for Rep {
    type Error = Error;

    fn try_from(conf: SockConfig) -> Result<Self, Self::Error> {
        Ok(Rep { inner: conf.build(zmq::REP)? })
    }
}

pub struct RepControlled {
    inner: ControlledSocket,
}

impl<H> ControlledStreamSocket<H> for RepControlled
where
    H: ControlHandler,
{
    fn socket(&self) -> &ControlledSocket {
        &self.inner
    }
}
