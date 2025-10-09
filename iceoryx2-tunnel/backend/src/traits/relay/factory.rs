// Copyright (c) 2025 Contributors to the Eclipse Foundation
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
use core::fmt::Debug;

use iceoryx2::service::{static_config::StaticConfig, Service};

use crate::traits::EventRelay;
use crate::traits::PublishSubscribeRelay;

/// Builder trait for constructing relay types provided by the backend
/// implementation.
pub trait RelayBuilder {
    /// The error type returned when relay creation fails
    type CreationError: Error;
    /// The type of relay that this builder creates
    type Relay: Debug;

    /// Consumes the builder and attempts to create a relay instance.
    ///
    /// # Returns
    /// - `Ok(Self::Relay)` if the relay was successfully created
    /// - `Err(Self::CreationError)` if creation failed
    fn create(self) -> Result<Self::Relay, Self::CreationError>;
}

/// Factory for creating relay builders  for supported messaging patterns.
pub trait RelayFactory<S: Service> {
    /// The publish-subscribe relay type that this factory creates
    type PublishSubscribeRelay: PublishSubscribeRelay<S>;
    /// The event relay type that this factory creates
    type EventRelay: EventRelay<S>;

    /// Builder type for creating publish-subscribe relays
    type PublishSubscribeBuilder<'config>: RelayBuilder<Relay = Self::PublishSubscribeRelay>
        + Debug
        + 'config
    where
        Self: 'config;

    /// Builder type for creating event relays
    type EventBuilder<'config>: RelayBuilder<Relay = Self::EventRelay> + Debug + 'config
    where
        Self: 'config;

    /// Creates a builder for publish-subscribe relays using the provided
    /// static configuration.
    fn publish_subscribe<'config>(
        &self,
        static_config: &'config StaticConfig,
    ) -> Self::PublishSubscribeBuilder<'config>
    where
        Self: 'config;

    /// Creates a builder for event relays using the provided static
    /// configuration.
    fn event<'config>(&self, static_config: &'config StaticConfig) -> Self::EventBuilder<'config>
    where
        Self: 'config;
}
