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

use std::rc::Rc;

use zmq;

use super::{StreamSocket, ZmqSocket};
use async::stream::{ControlHandler, ZmqControlledStream};

#[derive(ZmqSocket, SinkSocket, StreamSocket, CustomBuilder, CustomControlled)]
pub struct Sub {
    sock: Rc<zmq::Socket>,
}

impl Sub {
    pub fn new() -> SubBuilder {
        SubBuilder::new()
    }

    pub fn controlled<S>(controller: S) -> SubControlledBuilder
    where
        S: StreamSocket + ZmqSocket,
    {
        SubControlledBuilder::new(controller)
    }
}

pub enum SubCustomBuilder {
    Sock(Rc<zmq::Socket>),
    Fail(zmq::Error),
}

impl SubCustomBuilder {
    pub fn filter(self, filter: &[u8]) -> zmq::Result<Sub> {
        match self {
            SubCustomBuilder::Sock(sock) => {
                sock.set_subscribe(filter)?;

                Ok(Sub { sock })
            }
            SubCustomBuilder::Fail(e) => Err(e),
        }
    }
}

pub enum SubControlledCustomBuilder {
    Sock(Rc<zmq::Socket>, Rc<zmq::Socket>),
    Fail(zmq::Error),
}

impl SubControlledCustomBuilder {
    pub fn filter(self, filter: &[u8]) -> zmq::Result<SubControlled> {
        match self {
            SubControlledCustomBuilder::Sock(sock, controller) => {
                sock.set_subscribe(filter)?;

                Ok(SubControlled { sock, controller })
            }
            SubControlledCustomBuilder::Fail(e) => Err(e),
        }
    }
}
