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

use core::fmt::Debug;

use iceoryx2::service::static_config::StaticConfig;

// TODO: Rename to Ports?
/// Relays data over the transport.
///
/// A relay should be created for each messaging pattern.
pub trait Relay {
    type PropagationError: Debug;
    type IngestionError: Debug;

    fn propagate(&self, bytes: *const u8, len: usize) -> Result<(), Self::PropagationError>;
    fn ingest(
        &self,
        loan: &mut dyn FnMut(usize) -> (*mut u8, usize),
    ) -> Result<bool, Self::IngestionError>;
}

/// Builds a relay for a specific messaging pattern.
///
/// Allows for the transport to decide how to support
/// propagation of data on different messaging patterns.
pub trait RelayBuilder {
    type CreationError: Debug;
    type Relay: Relay;

    fn create(self) -> Result<Self::Relay, Self::CreationError>;
}

/// Factory for creating relay builders for different messaging patterns.
///
/// This trait defines the messaging patterns that a transport implementation
/// must support. Each builder method accepts a [`StaticConfig`] and returns
/// a builder that can create the corresponding relay.
///
/// The associated types use Generic Associated Types (GATs) to allow each
/// builder to have its own independent lifetime tied to the `StaticConfig`
/// it borrows. This means:
///
/// - Multiple builders can be created from the same factory
/// - Each builder can borrow a different `StaticConfig` with its own lifetime
/// - The factory (and any resources it holds) must outlive all builders it creates
pub trait RelayFactory {
    type PublishSubscribeRelay: Relay;
    type EventRelay: Relay;

    /// Builder for publish-subscribe messaging pattern.
    ///
    /// The `'config` lifetime parameter allows each call to `publish_subscribe()`
    /// to have its own independent lifetime tied to the borrowed `StaticConfig`.
    ///
    /// The `where Self: 'config` constraint ensures the factory and its resources
    /// outlive the builder.
    type PublishSubscribeBuilder<'config>: RelayBuilder<Relay = Self::PublishSubscribeRelay>
        + Debug
        + 'config
    where
        Self: 'config;

    /// Builder for event messaging pattern.
    ///
    /// The `'config` lifetime parameter allows each call to `event()`
    /// to have its own independent lifetime tied to the borrowed `StaticConfig`.
    ///
    /// The `where Self: 'config` constraint ensures the factory and its resources
    /// outlive the builder.
    type EventBuilder<'config>: RelayBuilder<Relay = Self::EventRelay> + Debug + 'config
    where
        Self: 'config;

    /// Creates a builder for the publish-subscribe messaging pattern.
    ///
    /// # Arguments
    ///
    /// * `static_config` - Configuration for the service. The returned builder
    ///   will borrow this config for its lifetime `'config`.
    ///
    /// # Lifetime
    ///
    /// The `where Self: 'config` constraint ensures that the factory (and any
    /// resources it holds, like a session) outlives the `static_config`
    /// and the returned builder.
    fn publish_subscribe<'config>(
        &self,
        static_config: &'config StaticConfig,
    ) -> Self::PublishSubscribeBuilder<'config>
    where
        Self: 'config;

    /// Creates a builder for the event messaging pattern.
    ///
    /// # Arguments
    ///
    /// * `static_config` - Configuration for the service. The returned builder
    ///   will borrow this config for its lifetime `'config`.
    ///
    /// # Lifetime
    ///
    /// The `where Self: 'config` constraint ensures that the factory (and any
    /// resources it holds, like a session) outlives the `static_config`
    /// and the returned builder.
    fn event<'config>(&self, static_config: &'config StaticConfig) -> Self::EventBuilder<'config>
    where
        Self: 'config;
}
