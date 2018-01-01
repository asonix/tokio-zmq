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

pub mod future;
pub mod sink;
pub mod stream;

use std::collections::VecDeque;

use zmq;

pub use self::future::{MultipartRequest, MultipartResponse};
pub use self::sink::MultipartSink;
pub use self::stream::{ControlledStream, ControlHandler, MultipartStream};

pub type Multipart = VecDeque<zmq::Message>;
