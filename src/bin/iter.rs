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
    let sock = context.socket(zmq::REP).unwrap();
    let zmq = ZmqREP::bind(&sock, "tcp://*:5560").unwrap();

    println!("Got zmq");

    for msg in zmq.incomming() {
        println!("Got something");

        let msg = match msg {
            Ok(msg) => msg,
            Err(err) => {
                println!("Error: '{}'", err);
                continue;
            }
        };

        let msg = match msg {
            Ok(msg) => msg,
            Err(err) => {
                let err = match String::from_utf8(err) {
                    Ok(err) => err,
                    _ => {
                        println!("Uknown ZMQ Error");
                        continue;
                    }
                };

                println!("Error: '{}'", err);
                continue;
            }
        };

        println!("Got Message: {}", msg);

        let _ = zmq.send("hey");
    }
}
