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

/// Defines the core functionality for tunnel backends utilizing an arbitrary
/// communication mechanism.
///
/// # Type Parameters
///
/// * `S` - The service type that implements the `Service` trait
///
/// # Associated Types
///
/// * `Config` - Configuration type for the backend
/// * `CreationError` - Error type returned during backend creation
/// * `Discovery` - Discovery implemenation using the backend mechanism
/// * `PublishSubscribeRelay` - Relay type for the publish-subscribe messaging pattern
/// * `EventRelay` - Relay type for the event messaging pattern
/// * `RelayFactory` - Factory type for creating relay instances
///
pub trait Backend<S: Service>: Sized {
    /// Configuration type for the backend initialization
    type Config: Default + Debug;

    /// Error type that can occur during backend creation
    type CreationError: Error;

    /// Discovery implentation for finding services using the backend mechanism
    type Discovery: Discovery + Debug;

    /// Relay implementation for the publish-subscribe messaging pattern
    type PublishSubscribeRelay: PublishSubscribeRelay<S> + Debug;

    /// Relay implementation for the event messaging pattern
    type EventRelay: EventRelay<S> + Debug;

    /// Factory type for creating relay instances
    type RelayFactory<'a>: RelayFactory<
            S,
            PublishSubscribeRelay = Self::PublishSubscribeRelay,
            EventRelay = Self::EventRelay,
        > + Debug
    where
        Self: 'a;

    /// Creates a new backend instance with the provided configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration parameters for initializing the backend
    ///
    /// # Returns
    ///
    /// Returns `Ok(Self)` with the created backend instance on success,
    /// or `Err(Self::CreationError)` if the backend creation fails.
    fn create(config: &Self::Config) -> Result<Self, Self::CreationError>;

    /// Returns a reference to the discovery implementations.
    fn discovery(&self) -> &impl Discovery;

    /// Creates a new relay factory instance.
    ///
    /// The relay factory is used to create specific relay builders for
    /// the supported messaging patterns.
    fn relay_builder(&self) -> Self::RelayFactory<'_>;
}
