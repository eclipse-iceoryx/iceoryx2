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

use core::error::Error;

use iceoryx2::service::Service;

use crate::relay::{EventRelay, PublishSubscribeRelay};
use crate::service_description::{EventDescription, PublishSubscribeDescription};

/// Configures and creates one relay.
pub trait RelayBuilder {
    type CreationError: Error;
    type Relay;

    /// Creates the relay.
    fn create(self) -> Result<Self::Relay, Self::CreationError>;
}

/// Creates the relays of a backend, each for a bridged service and the
/// description it is opened as on the opposing side.
pub trait RelayFactory<S: Service> {
    type RemoteDescription;
    type PublishSubscribeRelay: PublishSubscribeRelay<S>;
    type PublishSubscribeBuilder<'a>: RelayBuilder<Relay = Self::PublishSubscribeRelay> + 'a
    where
        Self: 'a;
    type EventRelay: EventRelay<S>;
    type EventBuilder<'a>: RelayBuilder<Relay = Self::EventRelay> + 'a
    where
        Self: 'a;

    /// Initiate the build for a publish-subscribe relay.
    fn publish_subscribe<'a>(
        &'a mut self,
        description: PublishSubscribeDescription<'a>,
        remote: &'a Self::RemoteDescription,
    ) -> Self::PublishSubscribeBuilder<'a>
    where
        Self: 'a;

    /// Initiate the build for an event relay.
    fn event<'a>(
        &'a mut self,
        description: EventDescription<'a>,
        remote: &'a Self::RemoteDescription,
    ) -> Self::EventBuilder<'a>
    where
        Self: 'a;
}
