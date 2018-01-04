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

//! This module defines all the socket wrapper types that can be used with Tokio.

use std::convert::TryFrom;

use zmq;

use socket::config::{PairConfig, SockConfig, SubConfig};
use prelude::*;
use socket::{ControlledSocket, Socket};
use error::Error;

/* -------------------------------------------------------------------------- */

/// The DEALER SocketType wrapper type.
///
/// Dealer implements StreamSocket and SinkSocket, and has an associated controlled variant.
#[derive(Clone)]
pub struct Dealer {
    inner: Socket,
}

impl Dealer {
    /// Construct a controled version of the Dealer socket.
    pub fn controlled<S>(self, control: S) -> DealerControlled
    where
        S: StreamSocket,
    {
        DealerControlled { inner: self.inner.controlled(control) }
    }
}

impl AsSocket for Dealer {
    fn socket(&self) -> &Socket {
        &self.inner
    }

    fn into_socket(self) -> Socket {
        self.inner
    }
}

impl StreamSocket for Dealer {}
impl SinkSocket for Dealer {}

impl<'a> TryFrom<SockConfig<'a>> for Dealer {
    type Error = Error;

    fn try_from(conf: SockConfig<'a>) -> Result<Self, Self::Error> {
        Ok(Dealer { inner: conf.build(zmq::DEALER)? })
    }
}

/// The controlled variant of Dealer
#[derive(Clone)]
pub struct DealerControlled {
    inner: ControlledSocket,
}

impl AsControlledSocket for DealerControlled {
    fn socket(&self) -> &ControlledSocket {
        &self.inner
    }
}

impl<H> ControlledStreamSocket<H> for DealerControlled
where
    H: ControlHandler,
{
}

impl ControlledSinkSocket for DealerControlled {}

/* -------------------------------------------------------------------------- */

/// The PAIR SocketType wrapper type.
///
/// Pair implements StreamSocket and SinkSocket, and has an associated controlled variant.
#[derive(Clone)]
pub struct Pair {
    inner: Socket,
}

impl Pair {
    /// Construct a controlled variant of the Pair socket
    pub fn controlled<S>(self, control: S) -> PairControlled
    where
        S: StreamSocket,
    {
        PairControlled { inner: self.inner.controlled(control) }
    }
}

impl AsSocket for Pair {
    fn socket(&self) -> &Socket {
        &self.inner
    }

    fn into_socket(self) -> Socket {
        self.inner
    }
}

impl StreamSocket for Pair {}
impl SinkSocket for Pair {}

impl<'a> TryFrom<PairConfig<'a>> for Pair {
    type Error = Error;

    fn try_from(conf: PairConfig<'a>) -> Result<Self, Self::Error> {
        Ok(Pair { inner: conf.build(zmq::PAIR)? })
    }
}

/// The controlled variant of Pair
#[derive(Clone)]
pub struct PairControlled {
    inner: ControlledSocket,
}

impl AsControlledSocket for PairControlled {
    fn socket(&self) -> &ControlledSocket {
        &self.inner
    }
}

impl<H> ControlledStreamSocket<H> for PairControlled
where
    H: ControlHandler,
{
}

impl ControlledSinkSocket for PairControlled {}

/* -------------------------------------------------------------------------- */

/// The PUB SocketType wrapper type
///
/// Pub implements SinkSocket.
#[derive(Clone)]
pub struct Pub {
    inner: Socket,
}

impl AsSocket for Pub {
    fn socket(&self) -> &Socket {
        &self.inner
    }

    fn into_socket(self) -> Socket {
        self.inner
    }
}

impl SinkSocket for Pub {}

impl<'a> TryFrom<SockConfig<'a>> for Pub {
    type Error = Error;

    fn try_from(conf: SockConfig<'a>) -> Result<Self, Self::Error> {
        Ok(Pub { inner: conf.build(zmq::PUB)? })
    }
}

/* -------------------------------------------------------------------------- */

/// The PULL SocketType wrapper type
///
/// Pull implements StreamSocket, and has an associated controlled variant.
#[derive(Clone)]
pub struct Pull {
    inner: Socket,
}

impl Pull {
    /// Construct a controlled variant of the Pull socket
    pub fn controlled<S>(self, control: S) -> PullControlled
    where
        S: StreamSocket,
    {
        PullControlled { inner: self.inner.controlled(control) }
    }
}

impl AsSocket for Pull {
    fn socket(&self) -> &Socket {
        &self.inner
    }

    fn into_socket(self) -> Socket {
        self.inner
    }
}

impl StreamSocket for Pull {}

impl<'a> TryFrom<SockConfig<'a>> for Pull {
    type Error = Error;

    fn try_from(conf: SockConfig<'a>) -> Result<Self, Self::Error> {
        Ok(Pull { inner: conf.build(zmq::PULL)? })
    }
}

/// The controlled variant of Pull
#[derive(Clone)]
pub struct PullControlled {
    inner: ControlledSocket,
}

impl AsControlledSocket for PullControlled {
    fn socket(&self) -> &ControlledSocket {
        &self.inner
    }
}

impl<H> ControlledStreamSocket<H> for PullControlled
where
    H: ControlHandler,
{
}

/* -------------------------------------------------------------------------- */

/// The PUSH SocketType wrapper type
///
/// Push implements SinkSocket.
#[derive(Clone)]
pub struct Push {
    inner: Socket,
}

impl AsSocket for Push {
    fn socket(&self) -> &Socket {
        &self.inner
    }

    fn into_socket(self) -> Socket {
        self.inner
    }
}

impl SinkSocket for Push {}

impl<'a> TryFrom<SockConfig<'a>> for Push {
    type Error = Error;

    fn try_from(conf: SockConfig<'a>) -> Result<Self, Self::Error> {
        Ok(Push { inner: conf.build(zmq::PUSH)? })
    }
}

/* -------------------------------------------------------------------------- */

/// The REP SocketType wrapper type
///
/// Rep implements StreamSocket and SinkSocket, and has an associated controlled variant.
#[derive(Clone)]
pub struct Rep {
    inner: Socket,
}

impl Rep {
    /// Construct a controlled variant of the Rep socket
    pub fn controlled<S>(self, control: S) -> RepControlled
    where
        S: StreamSocket,
    {
        RepControlled { inner: self.inner.controlled(control) }
    }
}

impl AsSocket for Rep {
    fn socket(&self) -> &Socket {
        &self.inner
    }

    fn into_socket(self) -> Socket {
        self.inner
    }
}

impl StreamSocket for Rep {}
impl SinkSocket for Rep {}

impl<'a> TryFrom<SockConfig<'a>> for Rep {
    type Error = Error;

    fn try_from(conf: SockConfig<'a>) -> Result<Self, Self::Error> {
        Ok(Rep { inner: conf.build(zmq::REP)? })
    }
}

/// The controlled variant of Rep
#[derive(Clone)]
pub struct RepControlled {
    inner: ControlledSocket,
}

impl AsControlledSocket for RepControlled {
    fn socket(&self) -> &ControlledSocket {
        &self.inner
    }
}

impl<H> ControlledStreamSocket<H> for RepControlled
where
    H: ControlHandler,
{
}

impl ControlledSinkSocket for RepControlled {}

/* -------------------------------------------------------------------------- */

/// The REQ SocketType wrapper type
///
/// Req implements FutureSocket.
#[derive(Clone)]
pub struct Req {
    inner: Socket,
}

impl AsSocket for Req {
    fn socket(&self) -> &Socket {
        &self.inner
    }

    fn into_socket(self) -> Socket {
        self.inner
    }
}

impl FutureSocket for Req {}

impl<'a> TryFrom<SockConfig<'a>> for Req {
    type Error = Error;

    fn try_from(conf: SockConfig<'a>) -> Result<Self, Self::Error> {
        Ok(Req { inner: conf.build(zmq::REQ)? })
    }
}

/* -------------------------------------------------------------------------- */

/// The ROUTER SocketType wrapper type
///
/// Router implements StreamSocket and SinkSocket, and has an associated controlled variant.
#[derive(Clone)]
pub struct Router {
    inner: Socket,
}

impl Router {
    /// Construct a controlled variant of the Router socket
    pub fn controlled<S>(self, control: S) -> RouterControlled
    where
        S: StreamSocket,
    {
        RouterControlled { inner: self.inner.controlled(control) }
    }
}

impl AsSocket for Router {
    fn socket(&self) -> &Socket {
        &self.inner
    }

    fn into_socket(self) -> Socket {
        self.inner
    }
}

impl StreamSocket for Router {}
impl SinkSocket for Router {}

impl<'a> TryFrom<SockConfig<'a>> for Router {
    type Error = Error;

    fn try_from(conf: SockConfig<'a>) -> Result<Self, Self::Error> {
        Ok(Router { inner: conf.build(zmq::ROUTER)? })
    }
}

/// The controlled variant of Router
#[derive(Clone)]
pub struct RouterControlled {
    inner: ControlledSocket,
}

impl AsControlledSocket for RouterControlled {
    fn socket(&self) -> &ControlledSocket {
        &self.inner
    }
}

impl<H> ControlledStreamSocket<H> for RouterControlled
where
    H: ControlHandler,
{
}

impl ControlledSinkSocket for RouterControlled {}

/* -------------------------------------------------------------------------- */

/// The SUB SocketType wrapper type
///
/// Sub implements StreamSocket, and has an associated controlled variant.
#[derive(Clone)]
pub struct Sub {
    inner: Socket,
}

impl Sub {
    /// Construct a controlled variant of the Sub socket
    pub fn controlled<S>(self, control: S) -> SubControlled
    where
        S: StreamSocket,
    {
        SubControlled { inner: self.inner.controlled(control) }
    }
}

impl AsSocket for Sub {
    fn socket(&self) -> &Socket {
        &self.inner
    }

    fn into_socket(self) -> Socket {
        self.inner
    }
}

impl StreamSocket for Sub {}

impl<'a> TryFrom<SubConfig<'a>> for Sub {
    type Error = Error;

    fn try_from(conf: SubConfig<'a>) -> Result<Self, Self::Error> {
        Ok(Sub { inner: conf.build(zmq::SUB)? })
    }
}

/// The controlled variant of Sub.
#[derive(Clone)]
pub struct SubControlled {
    inner: ControlledSocket,
}

impl AsControlledSocket for SubControlled {
    fn socket(&self) -> &ControlledSocket {
        &self.inner
    }
}

impl<H> ControlledStreamSocket<H> for SubControlled
where
    H: ControlHandler,
{
}

/* -------------------------------------------------------------------------- */

/// The XPUB SocketType wrapper type
///
/// Xpub implements StreamSocket and SinkSocket, and has an associated controlled variant.
#[derive(Clone)]
pub struct Xpub {
    inner: Socket,
}

impl Xpub {
    /// Construct a controlled variant of the Xpub socket
    pub fn controlled<S>(self, control: S) -> XpubControlled
    where
        S: StreamSocket,
    {
        XpubControlled { inner: self.inner.controlled(control) }
    }
}

impl AsSocket for Xpub {
    fn socket(&self) -> &Socket {
        &self.inner
    }

    fn into_socket(self) -> Socket {
        self.inner
    }
}

impl StreamSocket for Xpub {}
impl SinkSocket for Xpub {}

impl<'a> TryFrom<SockConfig<'a>> for Xpub {
    type Error = Error;

    fn try_from(conf: SockConfig<'a>) -> Result<Self, Self::Error> {
        Ok(Xpub { inner: conf.build(zmq::XPUB)? })
    }
}

/// The controlled variant of Xpub
#[derive(Clone)]
pub struct XpubControlled {
    inner: ControlledSocket,
}

impl AsControlledSocket for XpubControlled {
    fn socket(&self) -> &ControlledSocket {
        &self.inner
    }
}

impl<H> ControlledStreamSocket<H> for XpubControlled
where
    H: ControlHandler,
{
}

impl ControlledSinkSocket for XpubControlled {}

/* -------------------------------------------------------------------------- */

/// The XSUB SocketType wrapper type
///
/// Xsub implements StreamSocket and SinkSocket, and has an associated controlled variant.
#[derive(Clone)]
pub struct Xsub {
    inner: Socket,
}

impl Xsub {
    /// Construct a controlled variant of the Xsub socket
    pub fn control<S>(self, control: S) -> XsubControlled
    where
        S: StreamSocket,
    {
        XsubControlled { inner: self.inner.controlled(control) }
    }
}

impl AsSocket for Xsub {
    fn socket(&self) -> &Socket {
        &self.inner
    }

    fn into_socket(self) -> Socket {
        self.inner
    }
}

impl StreamSocket for Xsub {}
impl SinkSocket for Xsub {}

impl<'a> TryFrom<SockConfig<'a>> for Xsub {
    type Error = Error;

    fn try_from(conf: SockConfig<'a>) -> Result<Self, Self::Error> {
        Ok(Xsub { inner: conf.build(zmq::XSUB)? })
    }
}

/// The controlled variant of Xsub
#[derive(Clone)]
pub struct XsubControlled {
    inner: ControlledSocket,
}

impl AsControlledSocket for XsubControlled {
    fn socket(&self) -> &ControlledSocket {
        &self.inner
    }
}

impl<H> ControlledStreamSocket<H> for XsubControlled
where
    H: ControlHandler,
{
}

impl ControlledSinkSocket for XsubControlled {}
