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

use std::error::Error as StdError;
use std::fmt;
use std::io::Error as IoError;

use tokio_timer::TimerError;
use zmq::Error as ZmqError;

/// Defines the error type for Tokio ZMQ.
///
/// Errors here can come from two places, IO, and ZeroMQ. Most errors encountered in this
/// application are ZeroMQ errors, so `Error::Zmq(_)` is common, although we also need to catch IO
/// errors from Tokio's `PollEvented` creation and TokioFileUnix's File creation.
#[derive(Debug)]
pub enum Error {
    /// Stores ZeroMQ Errors
    Zmq(ZmqError),
    /// Stores PollEvented and File creation errors
    Io(IoError),
    /// Stores Tokio Timer errors
    Timer(TimerError),
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

impl From<TimerError> for Error {
    fn from(e: TimerError) -> Self {
        Error::Timer(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Zmq(ref e) => write!(f, "Error from ZeroMQ: {}", e),
            Error::Io(ref e) => write!(f, "Error creating file descriptor: {}", e),
            Error::Timer(ref e) => write!(f, "Error creating timer: {}", e),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Zmq(_) => "Error interacting with ZeroMQ",
            Error::Io(_) => "Error building socket",
            Error::Timer(_) => "Error creating timed stream",
        }
    }
}
