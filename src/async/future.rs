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

use futures::{Async, Future};
use futures::task::Context;
use mio::Ready;
use tokio::reactor::PollEvented2;
use tokio_file_unix::File;
use zmq;

use error::Error;
use file::ZmqFile;
use super::MsgPlace;
use message::Multipart;

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
/// #     let rep: Rep = Socket::builder(ctx, &core.handle())
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
    sock: Option<zmq::Socket>,
    file: Option<PollEvented2<File<ZmqFile>>>,
    multipart: Option<Multipart>,
}

impl MultipartRequest {
    pub fn new(sock: zmq::Socket, file: PollEvented2<File<ZmqFile>>, multipart: Multipart) -> Self {
        MultipartRequest {
            sock: Some(sock),
            file: Some(file),
            multipart: Some(multipart),
        }
    }

    pub(crate) fn take_socket(&mut self) -> Option<(zmq::Socket, PollEvented2<File<ZmqFile>>)> {
        if self.sock.is_some() && self.file.is_some() {
            self.sock
                .take()
                .and_then(|sock| self.file.take().map(|file| (sock, file)))
        } else {
            None
        }
    }

    pub(crate) fn give_socket(&mut self, sock: zmq::Socket, file: PollEvented2<File<ZmqFile>>) {
        self.sock = Some(sock);
        self.file = Some(file);
    }

    fn send(&mut self, cx: &mut Context) -> Result<Async<()>, Error> {
        while let Some(mut multipart) = self.multipart.take() {
            let msg = match multipart.pop_front() {
                Some(msg) => msg,
                None => {
                    self.multipart = None;
                    self.file
                        .as_ref()
                        .ok_or(Error::Reused)?
                        .clear_write_ready2(cx)?;
                    cx.waker().wake();
                    break;
                }
            };

            let place = if multipart.is_empty() {
                MsgPlace::Last
            } else {
                MsgPlace::Nth
            };

            debug!("MultipartRequest: sending: {:?}", msg.as_str());
            match self.send_msg(msg, &place, cx)? {
                None => {
                    if multipart.is_empty() {
                        break;
                    }
                }
                Some(msg) => {
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
        cx: &mut Context,
    ) -> Result<Option<zmq::Message>, Error> {
        let events = self.sock.as_ref().ok_or(Error::Reused)?.get_events()? as i16;

        if events & zmq::POLLOUT == 0 {
            self.file
                .as_ref()
                .ok_or(Error::Reused)?
                .clear_write_ready2(cx)?;

            cx.waker().wake();

            return Ok(Some(msg));
        }

        let flags = zmq::DONTWAIT | if *place == MsgPlace::Last {
            0
        } else {
            zmq::SNDMORE
        };

        match self.sock
            .as_ref()
            .ok_or(Error::Reused)?
            .send_msg(msg, flags)
        {
            Ok(_) => Ok(None),
            Err(e @ zmq::Error::EAGAIN) => {
                // return message in future
                debug!("MultipartRequest: EAGAIN");
                Err(e.into())
            }
            Err(e) => Err(e.into()),
        }
    }

    fn check_write(&mut self, cx: &mut Context) -> Result<bool, Error> {
        if let Async::Pending = self.file
            .as_ref()
            .ok_or(Error::Reused)?
            .poll_write_ready2(cx)?
        {
            // Get the events currently waiting on the socket
            let events = self.sock.as_ref().ok_or(Error::Reused)?.get_events()? as i16;
            if events & zmq::POLLOUT != 0 {
                // manually schedule a wakeup and procede
                self.file
                    .as_ref()
                    .ok_or(Error::Reused)?
                    .clear_write_ready2(cx)?;
                cx.waker().wake();
            } else {
                self.file
                    .as_ref()
                    .ok_or(Error::Reused)?
                    .clear_write_ready2(cx)?;
                return Ok(false);
            }
        }

        Ok(true)
    }
}

impl Future for MultipartRequest {
    type Item = (zmq::Socket, PollEvented2<File<ZmqFile>>);
    type Error = Error;

    fn poll(&mut self, cx: &mut Context) -> Result<Async<Self::Item>, Self::Error> {
        if self.check_write(cx)? {
            self.send(cx).and_then(|async| {
                Ok(match async {
                    Async::Ready(_) => {
                        let sock = self.sock.take().ok_or(Error::Reused)?;
                        let file = self.file.take().ok_or(Error::Reused)?;

                        Async::Ready((sock, file))
                    }
                    _ => Async::Pending,
                })
            })
        } else {
            Ok(Async::Pending)
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
/// #     let rep: Rep = Socket::builder(ctx, &core.handle())
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
    sock: Option<zmq::Socket>,
    file: Option<PollEvented2<File<ZmqFile>>>,
    multipart: Option<Multipart>,
}

impl MultipartResponse {
    pub fn new(sock: zmq::Socket, file: PollEvented2<File<ZmqFile>>) -> Self {
        MultipartResponse {
            sock: Some(sock),
            file: Some(file),
            multipart: None,
        }
    }

    pub(crate) fn take_socket(&mut self) -> Option<(zmq::Socket, PollEvented2<File<ZmqFile>>)> {
        if self.sock.is_some() && self.file.is_some() {
            self.sock
                .take()
                .and_then(|sock| self.file.take().map(|file| (sock, file)))
        } else {
            None
        }
    }

    pub(crate) fn give_socket(&mut self, sock: zmq::Socket, file: PollEvented2<File<ZmqFile>>) {
        self.sock = Some(sock);
        self.file = Some(file);
    }

    fn recv(&mut self, cx: &mut Context) -> Result<Async<Multipart>, Error> {
        let events = self.sock.as_ref().ok_or(Error::Reused)?.get_events()? as i16;

        if events & zmq::POLLIN == 0 {
            self.file
                .as_ref()
                .ok_or(Error::Reused)?
                .clear_read_ready2(cx, Ready::readable())?;

            cx.waker().wake();

            return Ok(Async::Pending);
        }

        let mut first = true;

        loop {
            match self.recv_msg()? {
                Async::Ready(msg) => {
                    first = false;
                    let mut multipart = self.multipart.take().unwrap_or_default();

                    let more = msg.get_more();

                    multipart.push_back(msg);

                    if !more {
                        return Ok(Async::Ready(multipart));
                    }

                    self.multipart = Some(multipart);
                }
                Async::Pending => {
                    if first {
                        return Ok(Async::Pending);
                    }
                }
            }
        }
    }

    fn recv_msg(&mut self) -> Result<Async<zmq::Message>, Error> {
        let mut msg = zmq::Message::new()?;

        match self.sock
            .as_ref()
            .ok_or(Error::Reused)?
            .recv(&mut msg, zmq::DONTWAIT)
        {
            Ok(_) => {
                debug!("MultipartResponse: received: {:?}", msg.as_str());
                Ok(Async::Ready(msg))
            }
            Err(zmq::Error::EAGAIN) => {
                debug!("MultipartResponse: EAGAIN");
                Ok(Async::Pending)
            }
            Err(e) => Err(e.into()),
        }
    }

    fn check_read(&mut self, cx: &mut Context) -> Result<bool, Error> {
        if let Async::Pending = self.file
            .as_ref()
            .ok_or(Error::Reused)?
            .poll_read_ready2(cx, Ready::readable())?
        {
            let events = self.sock.as_ref().ok_or(Error::Reused)?.get_events()? as i16;
            if events & zmq::POLLIN != 0 {
                // manually schedule a wakeup and procede
                self.file
                    .as_ref()
                    .ok_or(Error::Reused)?
                    .clear_read_ready2(cx, Ready::readable())?;
                cx.waker().wake();
            } else {
                self.file
                    .as_ref()
                    .ok_or(Error::Reused)?
                    .clear_read_ready2(cx, Ready::readable())?;
                return Ok(false);
            }
        }

        Ok(true)
    }
}

impl Future for MultipartResponse {
    type Item = (Multipart, zmq::Socket, PollEvented2<File<ZmqFile>>);
    type Error = Error;

    fn poll(&mut self, cx: &mut Context) -> Result<Async<Self::Item>, Self::Error> {
        if self.check_read(cx)? {
            self.recv(cx).and_then(|async| {
                Ok(match async {
                    Async::Ready(multipart) => {
                        let sock = self.sock.take().ok_or(Error::Reused)?;
                        let file = self.file.take().ok_or(Error::Reused)?;

                        Async::Ready((multipart, sock, file))
                    }
                    _ => Async::Pending,
                })
            })
        } else {
            Ok(Async::Pending)
        }
    }
}
