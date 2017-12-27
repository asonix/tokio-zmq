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

extern crate zmq;
extern crate zmq_futures;

use zmq_futures::ZmqREP;

fn main() {
    let context = zmq::Context::new();
    let sock = context.socket(zmq::REQ).unwrap();
    let zmq_sync = ZmqREP::connect(&sock, "tcp://localhost:5560").unwrap();

    for req in 0..5 {
        println!("sending: {}", req);
        zmq_sync.send("Hello").unwrap();

        let message = zmq_sync.incomming().next().unwrap().unwrap().unwrap();
        println!("Received reply {} {}", req, message);
    }
}
