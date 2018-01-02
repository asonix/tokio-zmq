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

use socket::config::PairConfig;
use prelude::*;
use socket::{ControlledSocket, Socket};
use error::Error;

pub struct Pair {
    inner: Socket,
}

impl Pair {
    pub fn controlled<S>(self, control: S) -> PairControlled
    where
        S: StreamSocket,
    {
        PairControlled { inner: self.inner.controlled(control) }
    }
}

impl AsSocket for Pair {
    fn socket(&self) -> &Socket {
        &self.inner
    }

    fn into_socket(self) -> Socket {
        self.inner
    }
}

impl StreamSocket for Pair {}
impl SinkSocket for Pair {}

impl<'a> TryFrom<PairConfig<'a>> for Pair {
    type Error = Error;

    fn try_from(conf: PairConfig<'a>) -> Result<Self, Self::Error> {
        Ok(Pair { inner: conf.build(zmq::PAIR)? })
    }
}

pub struct PairControlled {
    inner: ControlledSocket,
}

impl AsControlledSocket for PairControlled {
    fn socket(&self) -> &ControlledSocket {
        &self.inner
    }
}

impl<H> ControlledStreamSocket<H> for PairControlled
where
    H: ControlHandler,
{
}

impl ControlledSinkSocket for PairControlled {}
