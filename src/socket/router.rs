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
use prelude::*;
use socket::{ControlledSocket, Socket};
use error::Error;

pub struct Router {
    inner: Socket,
}

impl Router {
    pub fn controlled<S>(self, control: S) -> RouterControlled
    where
        S: StreamSocket,
    {
        RouterControlled { inner: self.inner.controlled(control) }
    }
}

impl AsSocket for Router {
    fn socket(&self) -> &Socket {
        &self.inner
    }

    fn into_socket(self) -> Socket {
        self.inner
    }
}

impl StreamSocket for Router {}
impl SinkSocket for Router {}

impl<'a> TryFrom<SockConfig<'a>> for Router {
    type Error = Error;

    fn try_from(conf: SockConfig<'a>) -> Result<Self, Self::Error> {
        Ok(Router { inner: conf.build(zmq::ROUTER)? })
    }
}

pub struct RouterControlled {
    inner: ControlledSocket,
}

impl AsControlledSocket for RouterControlled {
    fn socket(&self) -> &ControlledSocket {
        &self.inner
    }
}

impl<H> ControlledStreamSocket<H> for RouterControlled
where
    H: ControlHandler,
{
}

impl ControlledSinkSocket for RouterControlled {}
