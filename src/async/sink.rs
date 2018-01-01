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

use std::marker::PhantomData;
use std::rc::Rc;

use zmq;
use tokio_core::reactor::PollEvented;
use tokio_file_unix::File;
use futures::{Async, AsyncSink, Poll, Sink, StartSend};
use futures::task;

use super::Multipart;
use error::Error;
use ZmqFile;

#[derive(PartialEq)]
pub enum MsgPlace {
    Nth,
    Last,
}

pub struct MultipartSink<E>
where
    E: From<Error>,
{
    sock: Rc<zmq::Socket>,
    // Handles notifications to/from the event loop
    file: Rc<PollEvented<File<ZmqFile>>>,
    multipart: Option<Multipart>,
    phantom: PhantomData<E>,
}

impl<E> MultipartSink<E>
where
    E: From<Error>,
{
    pub fn new(sock: Rc<zmq::Socket>, file: Rc<PollEvented<File<ZmqFile>>>) -> Self {
        MultipartSink {
            sock: sock,
            file: file,
            multipart: None,
            phantom: PhantomData,
        }
    }

    fn send_msg(
        &mut self,
        msg: zmq::Message,
        place: MsgPlace,
    ) -> Result<AsyncSink<zmq::Message>, E> {
        debug!("MultipartSink: In send: {:?}", msg.as_str());
        // Get the events currently waiting on the socket
        let events = self.sock.get_events().map_err(Error::from)? as i16;
        debug!("MultipartSink: Got events");

        // Double check that the socket is ready for writing
        if events & zmq::POLLOUT == 0 {
            if events & zmq::POLLIN != 0 {
                self.file.need_read();
                debug!(
                    "MultipartSink: Events does not indicate POLLOUT: e{} != p{}",
                    events,
                    zmq::POLLOUT
                );
                return Ok(AsyncSink::NotReady(msg));
            } else {
                self.file.need_write();
                debug!(
                    "MultipartSink: Events does not indicate POLLOUT: e{} != p{}",
                    events,
                    zmq::POLLOUT
                );
                return Ok(AsyncSink::NotReady(msg));
            }
        }

        let flags = zmq::DONTWAIT |
            if place == MsgPlace::Last {
                debug!("MultipartSink: flags: DONTWAIT | 0");
                0
            } else {
                debug!("MultipartSink: flags: DONTWAIT | SNDMORE");
                zmq::SNDMORE
            };

        match self.sock.send_msg(msg, flags) {
            Ok(_) => {
                debug!("MultipartSink: Sent!");
                Ok(AsyncSink::Ready)
            }
            Err(zmq::Error::EAGAIN) => {
                /* I'd like to have the following code in place, but the Rust zmq high-level
                 * bindings don't return the message you just tried to send if it fails to send.
                 * For now, we bubble the error.
                 *
                 * // If EAGAIN, the socket is busy. This shouldn't happend because we have already
                 * // witnessed that the socket is ready for writing.
                 * Ok(AsyncSink::NotReady(msg))
                 */
                debug!("MultipartSink: EAGAIN");
                let e = zmq::Error::EAGAIN;
                let e: Error = e.into();
                Err(e.into())
            }
            Err(e) => {
                debug!("MultipartSink: Error: {}", e);
                let e: Error = e.into();
                Err(e.into())
            }
        }
    }

    fn flush(&mut self) -> Result<Async<()>, E> {
        debug!("MultipartSink: In flush");
        loop {
            if let Some(mut multipart) = self.multipart.take() {
                if let Some(curr_msg) = multipart.pop_front() {
                    debug!("MultipartSink: Sending current message");

                    let msg_place = if multipart.is_empty() {
                        MsgPlace::Last
                    } else {
                        MsgPlace::Nth
                    };

                    match self.send_msg(curr_msg, msg_place)? {
                        AsyncSink::Ready => {
                            debug!("MultipartSink: self.send success!");

                            if !multipart.is_empty() {
                                self.multipart = Some(multipart);
                                continue;
                                // return Ok(Async::NotReady);
                            }
                            break;
                        }
                        AsyncSink::NotReady(curr_msg) => {
                            debug!("MultipartSink: Failed send");
                            // If we couldn't send the current message, put it back and return NotReady
                            multipart.push_front(curr_msg);
                            self.multipart = Some(multipart);
                            continue;
                            // return Ok(Async::NotReady);
                        }
                    }
                } else {
                    self.multipart = None;
                    break;
                }
            } else {
                break;
            }
        }

        debug!("MultipartSink: Ready!");
        Ok(Async::Ready(()))
    }

    fn check_write(&mut self) -> Result<bool, E> {
        if let Async::NotReady = self.file.poll_write() {
            // Get the events currently waiting on the socket
            let events = self.sock.get_events().map_err(Error::from)? as i16;
            if events & zmq::POLLOUT != 0 {
                // manually schedule a wakeup and procede
                debug!("Write ready, but file doesn't think so");
                task::current().notify();
            } else {
                return Ok(false);
            }

            if events & zmq::POLLIN != 0 {
                self.file.need_read();
            }
        }

        Ok(true)
    }
}

impl<E> Sink for MultipartSink<E>
where
    E: From<Error>,
{
    type SinkItem = Multipart;
    type SinkError = E;

    fn start_send(
        &mut self,
        multipart: Self::SinkItem,
    ) -> StartSend<Self::SinkItem, Self::SinkError> {
        debug!("MultipartSink: start_send");
        if self.check_write()? {
            if self.multipart.is_none() {
                debug!("MultipartSink: Set multipart, ready!");
                self.multipart = Some(multipart);
                self.file.need_write();

                Ok(AsyncSink::Ready)
            } else {
                debug!("MultipartSink: not ready");
                match self.flush()? {
                    Async::Ready(()) => {
                    debug!("MultipartSink: Flushed, ready now");
                    self.multipart = Some(multipart);

                    Ok(AsyncSink::Ready)
                }
                    Async::NotReady => {
                        debug!("MultipartSink: still not ready");
                        Ok(AsyncSink::NotReady(multipart))
                    }
                }
            }
        } else {
            Ok(AsyncSink::NotReady(multipart))
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        debug!("MultipartSink: poll_complete");
        if self.check_write()? {
            self.flush()
        } else {
            Ok(Async::NotReady)
        }
    }
}
