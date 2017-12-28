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

#[derive(ZmqSocket, SinkSocket, StreamSocket, Builder)]
pub struct Rep {
    sock: Rc<zmq::Socket>,
}

impl Rep {
    pub fn new() -> RepBuilder {
        RepBuilder::new()
    }

    /*
    pub fn runner(
        &self,
    ) -> impl Future<
        Item = (impl Stream<Item = zmq::Message, Error = H::Error>, ZmqSink<H::Error>),
        Error = H::Error,
    > {
        let handler = self.handler.clone();

        self.stream()
            .map_err(H::Error::from)
            .and_then(move |msg| handler.call(msg.into()))
            .map(|msg| msg.into())
            .map_err(|e| e.into())
            .forward(self.sink())
    }
    */
}
