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

/// Builder pattern for constructing relay instances.
///
/// `RelayBuilder` follows the builder pattern to configure and create relay
/// instances. Builders are obtained from a `RelayFactory` and consumed upon
/// calling `create()`.
///
/// # Examples
///
/// Using a builder:
///
/// ```no_run
/// # use iceoryx2_tunnel_backend::traits::RelayBuilder;
/// # fn example<B: RelayBuilder>(builder: B) -> Result<B::Relay, B::CreationError> {
/// let relay = builder.create()?;
/// # Ok(relay)
/// # }
/// ```
///
/// Implementing a custom builder:
///
/// ```no_run
/// use iceoryx2_tunnel_backend::traits::RelayBuilder;
///
/// #[derive(Debug)]
/// struct MyRelayBuilder {
///     // Configuration
/// }
///
/// #[derive(Debug)]
/// struct MyRelay;
///
/// #[derive(Debug)]
/// struct MyError;
/// impl core::fmt::Display for MyError {
///     fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
///         write!(f, "creation failed")
///     }
/// }
/// impl core::error::Error for MyError {}
///
/// impl RelayBuilder for MyRelayBuilder {
///     type CreationError = MyError;
///     type Relay = MyRelay;
///
///     fn create(self) -> Result<Self::Relay, Self::CreationError> {
///         // Validate configuration and create relay
///         Ok(MyRelay)
///     }
/// }
/// ```
pub trait RelayBuilder {
    /// The error type returned when relay creation fails.
    type CreationError: Error;

    /// The type of relay that this builder creates.
    type Relay: Debug;

    /// Consumes the builder and attempts to create a relay instance.
    ///
    /// Validates the builder's configuration and constructs the relay. This
    /// method consumes the builder, ensuring it can only be used once.
    ///
    /// # Errors
    ///
    /// Returns an error if the operation cannot be completed. Implementations
    /// should provide error types that distinguish between failure modes
    fn create(self) -> Result<Self::Relay, Self::CreationError>;
}

/// Factory for creating relay builders for supported messaging patterns.
///
/// `RelayFactory` produces builders for the messaging patterns supported by
/// iceoryx2 (publish-subscribe and events). Each builder is configured with
/// the service's static configuration and can be further customized before
/// creating the final relay instance.
///
/// # Type Parameters
///
/// * `S` - The iceoryx2 service type
///
/// # Examples
///
/// Creating a publish-subscribe relay:
///
/// ```no_run
/// # use iceoryx2::service::{Service, static_config::StaticConfig};
/// # use iceoryx2_tunnel_backend::traits::{RelayFactory, RelayBuilder};
/// # fn example<'a, S: Service, F: RelayFactory<S>>(
/// #     factory: &F,
/// #     static_config: &'a StaticConfig
/// # ) -> Result<F::PublishSubscribeRelay, <<F as RelayFactory<S>>::PublishSubscribeBuilder<'a> as RelayBuilder>::CreationError> {
/// let builder = factory.publish_subscribe(static_config);
/// let relay = builder.create()?;
/// # Ok(relay)
/// # }
/// ```
///
/// Creating an event relay:
///
/// ```no_run
/// # use iceoryx2::service::{Service, static_config::StaticConfig};
/// # use iceoryx2_tunnel_backend::traits::{RelayFactory, RelayBuilder};
/// # fn example<'a, S: Service, F: RelayFactory<S>>(
/// #     factory: &F,
/// #     config: &'a StaticConfig
/// # ) -> Result<F::EventRelay, <<F as RelayFactory<S>>::EventBuilder<'a> as RelayBuilder>::CreationError> {
/// let builder = factory.event(config);
/// let relay = builder.create()?;
/// # Ok(relay)
/// # }
/// ```
pub trait RelayFactory<S: Service> {
    /// The publish-subscribe relay type that this factory creates.
    type PublishSubscribeRelay: PublishSubscribeRelay<S>;

    /// The event relay type that this factory creates.
    type EventRelay: EventRelay<S>;

    /// Builder type for creating publish-subscribe relays.
    type PublishSubscribeBuilder<'a>: RelayBuilder<Relay = Self::PublishSubscribeRelay> + Debug + 'a
    where
        Self: 'a;

    /// Builder type for creating event relays.
    type EventBuilder<'a>: RelayBuilder<Relay = Self::EventRelay> + Debug + 'a
    where
        Self: 'a;

    /// Creates a builder for publish-subscribe relays.
    ///
    /// Returns a builder configured with the service's static configuration.
    /// The builder can be further customized before calling `create()`.
    ///
    /// # Parameters
    ///
    /// * `static_config` - The service's static configuration containing
    ///   service name, messaging pattern details, and other metadata
    fn publish_subscribe<'a>(
        &self,
        static_config: &'a StaticConfig,
    ) -> Self::PublishSubscribeBuilder<'a>
    where
        Self: 'a;

    /// Creates a builder for event relays.
    ///
    /// Returns a builder configured with the service's static configuration.
    /// The builder can be further customized before calling `create()`.
    ///
    /// # Parameters
    ///
    /// * `static_config` - The service's static configuration containing
    ///   service name, event details, and other metadata
    fn event<'a>(&self, static_config: &'a StaticConfig) -> Self::EventBuilder<'a>
    where
        Self: 'a;
}
