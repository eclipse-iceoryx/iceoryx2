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

use iceoryx2::service::Service;

use crate::traits::{Discovery, EventRelay, PublishSubscribeRelay, RelayFactory};
use crate::types::wake::WakeHandle;

/// Core interface for tunnel backends that extend iceoryx2 over another
/// communication mechanism.
///
/// A [`Backend`] implementation provides the infrastructure for tunneling
/// iceoryx2 services over alternative transport layers (such as network
/// protocols, IPC mechanisms, or custom communication channels). It manages
/// service discovery and creates relays for different messaging patterns.
///
/// Backends are constructed via their associated [`Backend::Builder`] type,
/// obtained from [`Backend::builder()`].
///
/// # Type Parameters
///
/// * `S` - The [`iceoryx2::service::Service`] type being tunneled
///
/// # Architecture
///
/// A [`Backend`] implementation requires implementing several interconnected traits:
///
/// ```text
/// Backend
///   ├── Config
///   ├── Builder (BackendBuilder)
///   ├── Discovery
///   ├── RelayFactory
///   │   ├── PublishSubscribeRelay
///   │   │   └── PublishSubscribeBuilder
///   │   └── EventRelay
///   │       └── EventBuilder
///   └── Error
/// ```
///
/// Each component has specific responsibilities:
/// - **Config**: Backend-specific connection and initialization settings
/// - **Builder**: Constructs the [`Backend`] from its [`Backend::Config`]
/// - **Discovery**: Mechanisms to query the backend communication mechanism for remote services and announce local [`Service`]s
/// - **Relays**: Handle data transmission for each messaging pattern between the backend and iceoryx2
/// - **Factory**: Create [`RelayBuilder`](crate::traits::RelayBuilder) instances for specific relay types
/// - **Builders**: Construct relays with appropriate configuration
///
/// See individual trait documentation for [`BackendBuilder`], [`Discovery`],
/// [`PublishSubscribeRelay`], and [`EventRelay`] for implementation details.
pub trait Backend<S: Service>: Sized {
    /// Configuration type for the backend initialization
    type Config: Default + Debug;

    /// Error type that can occur during backend creation
    type CreationError: Error;

    /// Builder used to construct the backend.
    type Builder<'config>: BackendBuilder<S, Backend = Self, CreationError = Self::CreationError>
    where
        Self::Config: 'config;

    /// [`Discovery`] implementation for finding services using the [`Backend`]
    /// communication mechanism
    type Discovery: Discovery + Debug;

    /// [`PublishSubscribeRelay`] implementation for the publish-subscribe messaging pattern
    type PublishSubscribeRelay: PublishSubscribeRelay<S> + Debug;

    /// [`EventRelay`] implementation for the event messaging pattern
    type EventRelay: EventRelay<S> + Debug;

    /// Factory type for creating relay instances
    type RelayFactory<'a>: RelayFactory<
            S,
            PublishSubscribeRelay = Self::PublishSubscribeRelay,
            EventRelay = Self::EventRelay,
        > + Debug
    where
        Self: 'a;

    /// Returns a [`BackendBuilder`] bound to the provided configuration.
    fn builder(config: &Self::Config) -> Self::Builder<'_>;

    /// Returns a reference to the [`Discovery`] implementation.
    fn discovery(&self) -> &impl Discovery;

    /// Creates a new [`RelayFactory`] instance.
    ///
    /// The [`RelayFactory`] is used to create specific builder instances for
    /// relays for the supported messaging patterns.
    fn relay_builder(&self) -> Self::RelayFactory<'_>;
}

/// Builds a [`Backend`] from its [`Backend::Config`].
///
/// Each [`Backend`] has an associated [`BackendBuilder`] type accessed via
/// [`Backend::builder()`]. The builder is consumed by [`BackendBuilder::create`],
/// which performs any work required to bring the backend online.
pub trait BackendBuilder<S: Service> {
    /// The [`Backend`] this builder constructs.
    type Backend: Backend<S>;

    /// Error type returned by [`BackendBuilder::create`].
    type CreationError: Error;

    /// Consumes the builder, producing a configured [`Backend`].
    fn create(self) -> Result<Self::Backend, Self::CreationError>;
}

/// Opt-in capability for backends that signal a [`WakeHandle`] when
/// new data is ready to propagate. Polling-only backends must not implement it.
pub trait ReactiveBackendBuilder<S: Service>: BackendBuilder<S> {
    /// Configures the builder to produce a [`Backend`] that signals `wake`
    /// whenever it has new data ready to be propagated.
    fn reactive(self, wake: WakeHandle) -> Self;
}
