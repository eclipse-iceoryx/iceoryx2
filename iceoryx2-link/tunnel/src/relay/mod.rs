// Copyright (c) 2026 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

pub mod event;
pub mod publish_subscribe;

use core::error::Error;
use core::marker::PhantomData;

use iceoryx2::service::Service;
use iceoryx2_link_backend::description::{
    EventDescription, PublishSubscribeDescription, ServiceDescriptor,
};
use iceoryx2_link_backend::relay::RelayFactory;

use iceoryx2_link_carrier::Carrier;

pub struct Factory<'a, S: Service, C: Carrier> {
    carrier: &'a C,
    _service: PhantomData<S>,
}

impl<'a, S: Service, C: Carrier> Factory<'a, S, C> {
    pub(crate) fn new(carrier: &'a C) -> Self {
        Self {
            carrier,
            _service: PhantomData,
        }
    }
}

impl<'a, S: Service, C: Carrier> RelayFactory<S> for Factory<'a, S, C> {
    type RemoteDescription = ServiceDescriptor;
    type PublishSubscribeRelay = publish_subscribe::Relay<S, C::Channel>;
    type PublishSubscribeBuilder<'b>
        = publish_subscribe::Builder<'b, S, C>
    where
        Self: 'b;
    type EventRelay = event::Relay<S, C::Channel>;
    type EventBuilder<'b>
        = event::Builder<'b, S, C>
    where
        Self: 'b;

    fn publish_subscribe<'b>(
        &self,
        description: PublishSubscribeDescription<'b>,
        descriptor: &'b ServiceDescriptor,
    ) -> Self::PublishSubscribeBuilder<'b>
    where
        Self: 'b,
    {
        publish_subscribe::Builder::new(self.carrier, description, descriptor)
    }

    fn event<'b>(
        &self,
        description: EventDescription<'b>,
        descriptor: &'b ServiceDescriptor,
    ) -> Self::EventBuilder<'b>
    where
        Self: 'b,
    {
        event::Builder::new(self.carrier, description, descriptor)
    }
}

#[derive(Debug)]
pub enum CreationError<E> {
    /// The carrier could not open the channel.
    Channel(E),
}

impl<E: core::fmt::Debug> core::fmt::Display for CreationError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl<E: Error> Error for CreationError<E> {}

impl<E> From<E> for CreationError<E> {
    fn from(error: E) -> Self {
        Self::Channel(error)
    }
}

#[derive(Debug)]
pub enum SendError<E> {
    Channel(E),
}

impl<E: core::fmt::Debug> core::fmt::Display for SendError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SendError::{self:?}")
    }
}

impl<E: Error> Error for SendError<E> {}

impl<E> From<E> for SendError<E> {
    fn from(error: E) -> Self {
        Self::Channel(error)
    }
}

#[derive(Debug)]
pub enum ReceiveError<E> {
    Channel(E),
    /// The local publisher could not provide a sample.
    Loan,
    /// The frame does not fit the service's description.
    Malformed,
}

impl<E: core::fmt::Debug> core::fmt::Display for ReceiveError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ReceiveError::{self:?}")
    }
}

impl<E: Error> Error for ReceiveError<E> {}

impl<E> From<E> for ReceiveError<E> {
    fn from(error: E) -> Self {
        Self::Channel(error)
    }
}
