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

/// Core interface for tunnel backends that extend iceoryx2 over another
/// communication mechanism.
///
/// A `Backend` implementation provides the infrastructure for tunneling
/// iceoryx2 services over alternative transport layers (such as network
/// protocols, IPC mechanisms, or custom communication channels). It manages
/// service discovery and creates relays for different messaging patterns.
///
/// # Type Parameters
///
/// * `S` - The iceoryx2 service type being tunneled
///
/// # Architecture
///
/// A backend implementation requires implementing several interconnected traits:
///
/// ```text
/// MyBackend (Backend trait)
///   ├── MyConfig (configuration)
///   ├── MyDiscovery (Discovery trait)
///   ├── MyRelayFactory (RelayFactory trait)
///   │   ├── MyPublishSubscribeRelay (PublishSubscribeRelay trait)
///   │   │   └── MyPublishSubscribeBuilder (RelayBuilder trait)
///   │   └── MyEventRelay (EventRelay trait)
///   │       └── MyEventBuilder (RelayBuilder trait)
///   └── MyError (error types)
/// ```
///
/// Each component has specific responsibilities:
/// - **Config**: Backend-specific connection and initialization settings
/// - **Discovery**: Query the backend for available services
/// - **Relays**: Handle actual data transmission for each messaging pattern
/// - **Builders**: Construct relays with appropriate configuration
///
/// # Example: Basic Backend Structure
///
/// ```ignore
/// use iceoryx2::service::ipc::Service;
/// use iceoryx2_tunnel_backend::traits::Backend;
///
/// struct MyBackend {
///     // Your transport-specific state, e.g.:
///     // connection: TcpStream,
///     // discovery: ServiceRegistry,
/// }
///
/// impl Backend<Service> for MyBackend {
///     // ... associated types ...
///     
///     fn create(config: &Self::Config) -> Result<Self, Self::CreationError> {
///         // Establish connection to your backend transport
///         // let connection = TcpStream::connect(&config.endpoint)?;
///         
///         // Initialize your backend with the connection
///         // Ok(Self { connection, ... })
///         # unimplemented!()
///     }
///     
///     // ... implement discovery() and relay_builder() ...
/// }
/// ```
///
/// See individual trait documentation for [`Discovery`], [`PublishSubscribeRelay`],
/// and [`EventRelay`] for implementation details.
pub trait Backend<S: Service>: Sized {
    /// Configuration type for the backend initialization
    type Config: Default + Debug;

    /// Error type that can occur during backend creation
    type CreationError: Error;

    /// Discovery implementation for finding services using the backend
    /// communication mechanism
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
    fn create(config: &Self::Config) -> Result<Self, Self::CreationError>;

    /// Returns a reference to the [`Discovery`] implementation.
    fn discovery(&self) -> &impl Discovery;

    /// Creates a new relay factory instance.
    ///
    /// The relay factory is used to create specific relay builders for
    /// the supported messaging patterns.
    fn relay_builder(&self) -> Self::RelayFactory<'_>;
}
