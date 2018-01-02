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

use socket::config::SubConfig;
use prelude::*;
use socket::{ControlledSocket, Socket};
use error::Error;

pub struct Sub {
    inner: Socket,
}

impl Sub {
    pub fn controlled<S>(self, control: S) -> SubControlled
    where
        S: StreamSocket,
    {
        SubControlled { inner: self.inner.controlled(control) }
    }
}

impl AsSocket for Sub {
    fn socket(&self) -> &Socket {
        &self.inner
    }

    fn into_socket(self) -> Socket {
        self.inner
    }
}

impl StreamSocket for Sub {}

impl<'a> TryFrom<SubConfig<'a>> for Sub {
    type Error = Error;

    fn try_from(conf: SubConfig<'a>) -> Result<Self, Self::Error> {
        Ok(Sub { inner: conf.build(zmq::SUB)? })
    }
}

pub struct SubControlled {
    inner: ControlledSocket,
}

impl AsControlledSocket for SubControlled {
    fn socket(&self) -> &ControlledSocket {
        &self.inner
    }
}

impl<H> ControlledStreamSocket<H> for SubControlled
where
    H: ControlHandler,
{
}
