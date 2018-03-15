use std::mem::swap;

use futures::{Async, Sink, Stream};
use futures::task::Context;
use tokio_file_unix::File;
use tokio::reactor::PollEvented2;
use zmq;

use async::sink::MultipartSink;
use async::stream::MultipartStream;
use error::Error;
use file::ZmqFile;
use message::Multipart;

pub struct MultipartSinkStream {
    inner: SinkStreamState,
}

enum SinkStreamState {
    Sink(MultipartSink),
    Stream(MultipartStream),
    Both(
        MultipartSink,
        MultipartStream,
        zmq::Socket,
        PollEvented2<File<ZmqFile>>,
    ),
    Ready(zmq::Socket, PollEvented2<File<ZmqFile>>),
    Polling,
}

impl MultipartSinkStream {
    pub fn new(sock: zmq::Socket, file: PollEvented2<File<ZmqFile>>) -> Self {
        MultipartSinkStream {
            inner: SinkStreamState::Ready(sock, file),
        }
    }

    fn polling(&mut self) -> SinkStreamState {
        let mut state = SinkStreamState::Polling;

        swap(&mut self.inner, &mut state);

        state
    }

    fn poll_sink(
        &mut self,
        mut sink: MultipartSink,
        stream: Option<MultipartStream>,
        cx: &mut Context,
    ) -> Result<Async<()>, Error> {
        match sink.poll_flush(cx)? {
            Async::Ready(_) => match sink.take_socket() {
                Some((sock, file)) => {
                    debug!("Released sink");
                    match stream {
                        Some(mut stream) => {
                            stream.give_socket(sock, file);
                            self.inner = SinkStreamState::Stream(stream);
                        }
                        None => {
                            self.inner = SinkStreamState::Ready(sock, file);
                        }
                    }
                    Ok(Async::Ready(()))
                }
                None => Err(Error::Sink),
            },
            Async::Pending => {
                match stream {
                    Some(mut stream) => match sink.take_socket() {
                        Some((sock, file)) => {
                            self.inner = SinkStreamState::Both(sink, stream, sock, file);
                        }
                        None => {
                            return Err(Error::Sink);
                        }
                    },
                    None => {
                        self.inner = SinkStreamState::Sink(sink);
                    }
                }

                Ok(Async::Pending)
            }
        }
    }

    fn poll_stream(
        &mut self,
        mut stream: MultipartStream,
        sink: Option<MultipartSink>,
        cx: &mut Context,
    ) -> Result<Async<Option<Multipart>>, Error> {
        match stream.poll_next(cx)? {
            Async::Ready(item) => match stream.take_socket() {
                Some((sock, file)) => {
                    debug!("Released stream");
                    match sink {
                        Some(mut sink) => {
                            sink.give_socket(sock, file);
                            self.inner = SinkStreamState::Sink(sink);
                        }
                        None => {
                            self.inner = SinkStreamState::Ready(sock, file);
                        }
                    }
                    Ok(Async::Ready(item))
                }
                None => Err(Error::Stream),
            },
            Async::Pending => {
                match sink {
                    Some(mut sink) => match stream.take_socket() {
                        Some((sock, file)) => {
                            self.inner = SinkStreamState::Both(sink, stream, sock, file);
                        }
                        None => {
                            return Err(Error::Stream);
                        }
                    },
                    None => {
                        self.inner = SinkStreamState::Stream(stream);
                    }
                }

                Ok(Async::Pending)
            }
        }
    }
}

impl Sink for MultipartSinkStream {
    type SinkItem = Multipart;
    type SinkError = Error;

    fn start_send(&mut self, multipart: Self::SinkItem) -> Result<(), Self::SinkError> {
        debug!("Called start_send");
        match self.polling() {
            SinkStreamState::Ready(sock, file) => {
                let mut sink = MultipartSink::new(sock, file);
                sink.start_send(multipart)?;
                self.inner = SinkStreamState::Sink(sink);
                debug!("Created sink");
                Ok(())
            }
            SinkStreamState::Stream(mut stream) => match stream.take_socket() {
                Some((sock, file)) => {
                    let mut sink = MultipartSink::new(sock, file);
                    sink.start_send(multipart)?;
                    match sink.take_socket() {
                        Some((sock, file)) => {
                            self.inner = SinkStreamState::Both(sink, stream, sock, file);
                            debug!("Created sink");
                            Ok(())
                        }
                        None => Err(Error::Sink),
                    }
                }
                None => Err(Error::Sink),
            },
            _ => Err(Error::Sink),
        }
    }

    fn poll_ready(&mut self, cx: &mut Context) -> Result<Async<()>, Self::SinkError> {
        debug!("Called poll_ready");
        self.poll_flush(cx)
    }

    fn poll_flush(&mut self, cx: &mut Context) -> Result<Async<()>, Self::SinkError> {
        debug!("Called poll_flush");
        match self.polling() {
            SinkStreamState::Ready(sock, file) => {
                self.inner = SinkStreamState::Ready(sock, file);
                Ok(Async::Ready(()))
            }
            SinkStreamState::Sink(sink) => self.poll_sink(sink, None, cx),
            SinkStreamState::Stream(stream) => {
                self.inner = SinkStreamState::Stream(stream);
                Ok(Async::Ready(()))
            }
            SinkStreamState::Both(mut sink, stream, sock, file) => {
                sink.give_socket(sock, file);
                self.poll_sink(sink, Some(stream), cx)
            }
            SinkStreamState::Polling => Err(Error::Sink),
        }
    }

    fn poll_close(&mut self, cx: &mut Context) -> Result<Async<()>, Self::SinkError> {
        debug!("Called poll_close");
        self.poll_flush(cx)
    }
}

impl Stream for MultipartSinkStream {
    type Item = Multipart;
    type Error = Error;

    fn poll_next(&mut self, cx: &mut Context) -> Result<Async<Option<Multipart>>, Self::Error> {
        match self.polling() {
            SinkStreamState::Ready(sock, file) => {
                let stream = MultipartStream::new(sock, file);
                debug!("Created stream");
                self.poll_stream(stream, None, cx)
            }
            SinkStreamState::Sink(mut sink) => match sink.take_socket() {
                Some((sock, file)) => {
                    let stream = MultipartStream::new(sock, file);
                    debug!("Created stream");
                    self.poll_stream(stream, Some(sink), cx)
                }
                None => Err(Error::Stream),
            },
            SinkStreamState::Both(sink, mut stream, sock, file) => {
                stream.give_socket(sock, file);
                self.poll_stream(stream, Some(sink), cx)
            }
            SinkStreamState::Stream(stream) => self.poll_stream(stream, None, cx),
            SinkStreamState::Polling => Err(Error::Stream),
        }
    }
}
