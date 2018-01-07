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

//! This module contains definitions for `MultipartRequest` and `MultipartResponse`, the two types that
//! implement `futures::Future`.

use std::rc::Rc;

use zmq;
use tokio_core::reactor::PollEvented;
use tokio_file_unix::File;
use futures::{Async, AsyncSink, Future, Poll};
use futures::task;

use error::Error;
use super::MsgPlace;
use message::Multipart;
use file::ZmqFile;

/// The `MultipartRequest` Future handles asynchronously sending data to a socket.
///
/// You shouldn't ever need to manually create one, but if you do, the following will suffice.
/// ### Example
/// ```rust
/// # #![feature(conservative_impl_trait)]
/// # #![feature(try_from)]
/// #
/// # extern crate zmq;
/// # extern crate futures;
/// # extern crate tokio_core;
/// # extern crate tokio_zmq;
/// #
/// # use std::rc::Rc;
/// # use std::convert::TryInto;
/// #
/// # use futures::Future;
/// # use tokio_core::reactor::Core;
/// # use tokio_zmq::prelude::*;
/// # use tokio_zmq::async::MultipartRequest;
/// # use tokio_zmq::{Error, Rep, Socket};
/// #
/// # fn main() {
/// #     get_sock();
/// # }
/// # fn get_sock() -> impl Future<Item = (), Error = Error> {
/// #     let core = Core::new().unwrap();
/// #     let ctx = Rc::new(zmq::Context::new());
/// #     let rep: Rep = Socket::create(ctx, &core.handle())
/// #         .bind("tcp://*:5567")
/// #         .try_into()
/// #         .unwrap();
/// #     let socket = rep.socket();
/// #     let sock = socket.inner_sock();
/// #     let file = socket.inner_file();
/// #     let msg = zmq::Message::from_slice(format!("Hey").as_bytes()).unwrap();
/// MultipartRequest::new(sock, file, msg.into()).and_then(|_| {
///     // succesfull request
///     # Ok(())
/// })
/// # }
/// ```
pub struct MultipartRequest {
    sock: Rc<zmq::Socket>,
    file: Rc<PollEvented<File<ZmqFile>>>,
    multipart: Option<Multipart>,
}

impl MultipartRequest {
    pub fn new(
        sock: Rc<zmq::Socket>,
        file: Rc<PollEvented<File<ZmqFile>>>,
        multipart: Multipart,
    ) -> Self {
        MultipartRequest {
            sock: sock,
            file: file,
            multipart: Some(multipart),
        }
    }

    fn send(&mut self) -> Poll<(), Error> {
        loop {
            debug!("MultipartRequest: loop");
            let mut multipart = match self.multipart.take() {
                Some(multipart) => multipart,
                None => {
                    debug!("MultipartRequest: breaking loop, no multipart");
                    break;
                }
            };

            let msg = match multipart.pop_front() {
                Some(msg) => msg,
                None => {
                    self.multipart = None;
                    debug!("MultipartRequest: breaking loop, empty multipart");
                    task::current().notify();
                    break;
                }
            };

            let place = if multipart.is_empty() {
                debug!("MultipartRequest: Last message");
                MsgPlace::Last
            } else {
                debug!("MultipartRequest: Nth message");
                MsgPlace::Nth
            };

            match self.send_msg(msg, &place)? {
                AsyncSink::Ready => {
                    debug!("MultipartRequest: Sent!");

                    if multipart.is_empty() {
                        debug!("MultipartRequest: breaking loop");
                        break;
                    }
                }
                AsyncSink::NotReady(msg) => {
                    multipart.push_front(msg);
                }
            }

            self.multipart = Some(multipart);
        }

        Ok(Async::Ready(()))
    }

    fn send_msg(
        &mut self,
        msg: zmq::Message,
        place: &MsgPlace,
    ) -> Result<AsyncSink<zmq::Message>, Error> {
        debug!("MultipartRequest: send_msg");
        let events = self.sock.get_events()? as i16;

        if events & zmq::POLLOUT == 0 {
            debug!("MultipartRequest: need_write()");
            self.file.need_write();

            return Ok(AsyncSink::NotReady(msg));
        }

        let flags = zmq::DONTWAIT | if *place == MsgPlace::Last {
            0
        } else {
            zmq::SNDMORE
        };

        match self.sock.send_msg(msg, flags) {
            Ok(_) => Ok(AsyncSink::Ready),
            Err(e @ zmq::Error::EAGAIN) => {
                // return message in future
                debug!("MultipartRequest: EAGAIN");
                Err(e.into())
            }
            Err(e) => Err(e.into()),
        }
    }

    fn check_write(&mut self) -> Result<bool, Error> {
        if let Async::NotReady = self.file.poll_write() {
            // Get the events currently waiting on the socket
            let events = self.sock.get_events()? as i16;
            if events & zmq::POLLOUT != 0 {
                // manually schedule a wakeup and procede
                debug!("MultipartRequest: Write ready, but file doesn't think so");
                task::current().notify();
            } else {
                return Ok(false);
            }

            if events & zmq::POLLIN != 0 {
                debug!("MultipartRequest: need_read()");
                self.file.need_read();
            }
        }

        Ok(true)
    }
}

impl Future for MultipartRequest {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        debug!("MultipartRequest: in poll");
        if self.check_write()? {
            self.send()
        } else {
            Ok(Async::NotReady)
        }
    }
}

/// The `MultipartResponse` Future handles asynchronously getting data from a socket.
///
/// You shouldn't ever need to manually create one, but if you do, the following will suffice.
/// ### Example
/// ```rust
/// # #![feature(conservative_impl_trait)]
/// # #![feature(try_from)]
/// #
/// # extern crate zmq;
/// # extern crate futures;
/// # extern crate tokio_core;
/// # extern crate tokio_zmq;
/// #
/// # use std::rc::Rc;
/// # use std::convert::TryInto;
/// #
/// # use futures::Future;
/// # use tokio_core::reactor::Core;
/// # use tokio_zmq::prelude::*;
/// # use tokio_zmq::async::{MultipartResponse};
/// # use tokio_zmq::{Error, Multipart, Rep, Socket};
/// #
/// # fn main() {
/// #     get_sock();
/// # }
/// # fn get_sock() -> impl Future<Item = Multipart, Error = Error> {
/// #     let core = Core::new().unwrap();
/// #     let ctx = Rc::new(zmq::Context::new());
/// #     let rep: Rep = Socket::create(ctx, &core.handle())
/// #         .bind("tcp://*:5567")
/// #         .try_into()
/// #         .unwrap();
/// #     let socket = rep.socket();
/// #     let sock = socket.inner_sock();
/// #     let file = socket.inner_file();
/// MultipartResponse::new(sock, file).and_then(|multipart| {
///     // handle multipart response
///     # Ok(multipart)
/// })
/// # }
/// ```
pub struct MultipartResponse {
    sock: Rc<zmq::Socket>,
    file: Rc<PollEvented<File<ZmqFile>>>,
    multipart: Option<Multipart>,
}

impl MultipartResponse {
    pub fn new(sock: Rc<zmq::Socket>, file: Rc<PollEvented<File<ZmqFile>>>) -> Self {
        MultipartResponse {
            sock: sock,
            file: file,
            multipart: None,
        }
    }

    fn recv(&mut self) -> Poll<Multipart, Error> {
        debug!("MultipartResponse: recv");
        let events = self.sock.get_events()? as i16;

        if events & zmq::POLLIN == 0 {
            debug!("MultipartResponse: need_read()");
            self.file.need_read();

            debug!("MultipartResponse: leaving recv");
            return Ok(Async::NotReady);
        }

        let mut first = true;

        loop {
            debug!("MultipartResponse: loop");
            match self.recv_msg()? {
                Async::Ready(msg) => {
                    first = false;
                    let mut multipart = self.multipart.take().unwrap_or_default();

                    let more = msg.get_more();

                    multipart.push_back(msg);

                    if !more {
                        debug!("MultipartResponse: Done receiving, returning multipart");
                        return Ok(Async::Ready(multipart));
                    }

                    debug!("MultipartResponse: Waiting on more");
                    self.multipart = Some(multipart);
                }
                Async::NotReady => {
                    if first {
                        debug!("MultipartResponse: leaving recv, not ready");
                        return Ok(Async::NotReady);
                    }
                }
            }
        }
    }

    fn recv_msg(&mut self) -> Poll<zmq::Message, Error> {
        debug!("MultipartResponse: recv_msg");
        let mut msg = zmq::Message::new()?;

        match self.sock.recv(&mut msg, zmq::DONTWAIT) {
            Ok(_) => {
                debug!("MultipartResponse: received: {:?}", msg.as_str());
                Ok(Async::Ready(msg))
            }
            Err(zmq::Error::EAGAIN) => {
                debug!("MultipartResponse: EAGAIN");
                Ok(Async::NotReady)
            }
            Err(e) => Err(e.into()),
        }
    }

    fn check_read(&mut self) -> Result<bool, Error> {
        if let Async::NotReady = self.file.poll_read() {
            let events = self.sock.get_events()? as i16;
            if events & zmq::POLLIN != 0 {
                // manually schedule a wakeup and procede
                debug!("MultipartResponse: Read ready, but file doesn't think so");
                task::current().notify();
            } else {
                return Ok(false);
            }

            if events & zmq::POLLOUT != 0 {
                self.file.need_write();
            }
        }

        Ok(true)
    }
}

impl Future for MultipartResponse {
    type Item = Multipart;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        debug!("MultipartResponse: In poll");
        if self.check_read()? {
            self.recv()
        } else {
            Ok(Async::NotReady)
        }
    }
}
