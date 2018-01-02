# Tokio ZMQ

This crate contains wrappers around ZeroMQ Concepts with Futures.

Currently Supported Sockets
 - REP
 - REQ
 - PUB
 - SUB
 - PUSH
 - PULL
 - XPUB
 - XSUB
 - PAIR

See the [examples folder](https://github.com/asonix/zmq-futures/tree/master/examples) for usage examples.

### Getting Started

Add the following to your Cargo.toml
```toml
zmq = "0.8"
tokio-zmq = "0.1.0"
futures = "0.1"
tokio-core = "0.1"
```

In your application:
```rust
use std::rc::Rc;
use std::convert::TryInto;

use futures::Stream;
use tokio_core::reactor::Core;
use tokio_zmq::prelude::*;
use tokio_zmq::{Socket, Error};
use tokio_zmq::Rep; // the socket type you want

fn main() {
  let mut core = Core::new().unwrap();
  let handle = core.handle();
  let context = Rc::new(zmq::Context::new());
  let rep: Rep = Socket::new(context, handle)
      .bind("tcp://*:5560")
      .try_into()
      .unwrap()

  let runner = rep.stream()
      .and_then(|multipart| {
          // handle the multipart (VecDeque<zmq::Message>)
          // This example simply echos the incoming data back to the client.
          Ok(multipart)
      })
      .forward(rep.sink::<Error>());

  core.run(runner).unwrap();
}
```

### Running the examples
The `req.rs` and `rep.rs` examples are designed to be used together. The `rep` example starts a server with a REP socket, and the `req` example queries that server with a REQ socket.

The `zpub.rs` and `sub.rs` examples should be used togheter. `zpub` produces values that `sub` consumes.

The `push.rs`, `pull_push.rs`, and `pull.rs` files should be used together. `push` produces values, which are relayed by `pull_push` to `pull`, which consumes them and sends a stop signal to itself and to `pull_push`.

### Contributing
Feel free to open issues for anything you find an issue with. Please note that any contributed code will be licensed under the GPLv3.

### License

Copyright © 2017 Riley Trautman

Tokio ZMQ is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Tokio ZMQ is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details. This file is part of Tokio ZMQ.

You should have received a copy of the GNU General Public License along with Tokio ZMQ. If not, see [http://www.gnu.org/licenses/](http://www.gnu.org/licenses/).
