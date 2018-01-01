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

use std::io::Error as IoError;
use zmq::Error as ZmqError;

#[derive(Debug)]
pub enum Error {
    Zmq(ZmqError),
    Io(IoError),
}

impl From<ZmqError> for Error {
    fn from(e: ZmqError) -> Self {
        Error::Zmq(e)
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Error::Io(e)
    }
}
