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
/// [`RelayBuilder`] follows the builder pattern to configure and create relay
/// instances. [`RelayBuilder`] instances are obtained from a [`RelayFactory`] and consumed upon
/// calling [`RelayBuilder::create()`].
///
/// # Examples
///
/// Building a relay with a [`RelayBuilder`]:
///
/// ```no_run
/// # use iceoryx2_tunnel_backend::traits::RelayBuilder;
/// # fn example<B: RelayBuilder>(builder: B) -> Result<B::Relay, B::CreationError> {
/// let relay = builder.create()?;
/// # Ok(relay)
/// # }
/// ```
///
/// Implementing a [`RelayBuilder`]:
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

    /// Consumes the [`RelayBuilder`] and attempts to create a [`RelayBuilder::Relay`] instance.
    ///
    /// Validates the [`RelayBuilder`]'s configuration and constructs the [`RelayBuilder::Relay`]. This
    /// method consumes the [`RelayBuilder`], ensuring it can only be used once.
    ///
    /// # Returns
    ///
    /// A relay that can communicate over the backend communication mechanism
    fn create(self) -> Result<Self::Relay, Self::CreationError>;
}

/// Factory for creating relay builders for supported messaging patterns.
///
/// [`RelayFactory`] produces [`RelayBuilder`]s for the supported [`MessagingPattern`](iceoryx2::service::messaging_pattern::MessagingPattern)s.
/// Each [`RelayBuilder`] is configured with the [`Service`]'s [`StaticConfig`] and can be further customized before
/// creating the relay.
///
/// # Type Parameters
///
/// * `S` - The iceoryx2 [`Service`]
///
/// # Examples
///
/// Creating a
/// [`MessagingPattern::PublishSubscribe`](iceoryx2::service::messaging_pattern::MessagingPattern::PublishSubscribe) relay:
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
/// Creating an [`MessagingPattern::Event`](iceoryx2::service::messaging_pattern::MessagingPattern::Event) relay:
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
    /// The [`PublishSubscribe`](iceoryx2::service::messaging_pattern::MessagingPattern::PublishSubscribe)
    /// to be built by [`RelayBuilder`]s created by the [`RelayFactory`]
    type PublishSubscribeRelay: PublishSubscribeRelay<S>;

    /// The [`Event`](iceoryx2::service::messaging_pattern::MessagingPattern::Event)
    /// to be built by [`RelayBuilder`]s created by the [`RelayFactory`]
    type EventRelay: EventRelay<S>;

    /// [RelayBuilder] type for creating [`PublishSubscribe`](iceoryx2::service::messaging_pattern::MessagingPattern::PublishSubscribe)
    /// relays.
    type PublishSubscribeBuilder<'a>: RelayBuilder<Relay = Self::PublishSubscribeRelay> + Debug + 'a
    where
        Self: 'a;

    /// [RelayBuilder] type for creating [`Event`](iceoryx2::service::messaging_pattern::MessagingPattern::Event)
    /// relays.
    type EventBuilder<'a>: RelayBuilder<Relay = Self::EventRelay> + Debug + 'a
    where
        Self: 'a;

    /// Creates a [`RelayBuilder`] for [`PublishSubscribe`](iceoryx2::service::messaging_pattern::MessagingPattern::PublishSubscribe)
    /// relays.
    ///
    /// # Parameters
    ///
    /// * `static_config` - The [`Service`]'s [`StaticConfig`] for which a builder will be created
    ///
    /// # Returns
    ///
    /// A [`RelayBuilder`] configured with the [`Service`]'s [`StaticConfig`].
    /// The [`RelayBuilder`] can be further customized before calling [`RelayBuilder::create()`].
    ///
    fn publish_subscribe<'a>(
        &self,
        static_config: &'a StaticConfig,
    ) -> Self::PublishSubscribeBuilder<'a>
    where
        Self: 'a;

    /// Creates a [`RelayBuilder`] for [`Event`](iceoryx2::service::messaging_pattern::MessagingPattern::Event)
    /// relays.
    ///
    /// # Parameters
    ///
    /// * `static_config` - The [`Service`]'s [`StaticConfig`] for which a builder will be created
    ///
    /// # Returns
    ///
    /// A [`RelayBuilder`] configured with the [`Service`]'s [`StaticConfig`].
    /// The [`RelayBuilder`] can be further customized before calling [`RelayBuilder::create()`].
    fn event<'a>(&self, static_config: &'a StaticConfig) -> Self::EventBuilder<'a>
    where
        Self: 'a;
}
