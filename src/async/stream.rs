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

use std::rc::Rc;

use zmq;
use tokio_core::reactor::PollEvented;
use tokio_file_unix::File;
use futures::{Async, Future, Poll, Stream};

use async::future::MultipartResponse;
use error::Error;
use super::Multipart;
use ZmqFile;

pub struct MultipartStream {
    response: Option<MultipartResponse>,
    // To read data
    sock: Rc<zmq::Socket>,
    // Handles notifications to/from the event loop
    file: Rc<PollEvented<File<ZmqFile>>>,
}

impl MultipartStream {
    pub fn new(sock: Rc<zmq::Socket>, file: Rc<PollEvented<File<ZmqFile>>>) -> Self {
        MultipartStream {
            response: None,
            sock: sock,
            file: file,
        }
    }

    fn poll_response(&mut self, mut response: MultipartResponse) -> Poll<Option<Multipart>, Error> {
        match response.poll()? {
            Async::Ready(item) => Ok(Async::Ready(Some(item))),
            Async::NotReady => {
                self.response = Some(response);
                Ok(Async::NotReady)
            }
        }
    }
}

impl Stream for MultipartStream {
    type Item = Multipart;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Multipart>, Error> {
        if let Some(response) = self.response.take() {
            self.poll_response(response)
        } else {
            let response = MultipartResponse::new(Rc::clone(&self.sock), Rc::clone(&self.file));
            self.poll_response(response)
        }
    }
}

pub trait ControlHandler {
    fn should_stop(&self, multipart: Multipart) -> bool;
}

pub struct ControlledStream<H>
where
    H: ControlHandler,
{
    stream: MultipartStream,
    control: MultipartStream,
    handler: H,
}

impl<H> ControlledStream<H>
where
    H: ControlHandler,
{
    pub fn new(
        sock: Rc<zmq::Socket>,
        file: Rc<PollEvented<File<ZmqFile>>>,
        control_sock: Rc<zmq::Socket>,
        control_file: Rc<PollEvented<File<ZmqFile>>>,
        handler: H,
    ) -> Self {
        ControlledStream {
            stream: MultipartStream::new(sock, file),
            control: MultipartStream::new(control_sock, control_file),
            handler: handler,
        }
    }
}

impl<H> Stream for ControlledStream<H>
where
    H: ControlHandler,
{
    type Item = Multipart;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Multipart>, Error> {
        let stop = match self.control.poll()? {
            Async::NotReady => false,
            Async::Ready(None) => false,
            Async::Ready(Some(multipart)) => self.handler.should_stop(multipart),
        };

        if stop {
            Ok(Async::Ready(None))
        } else {
            self.stream.poll()
        }
    }
}
