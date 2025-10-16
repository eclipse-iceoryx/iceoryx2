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

use iceoryx2::service::static_config::StaticConfig;

/// Service discovery interface for discoverying and announcing
/// [`Service`](iceoryx2::service::Service)s over the [`Backend`](crate::traits::Backend)
/// communication mechanism.
///
/// [`Discovery`] enables mechansms to instrospect available iceoryx2
/// [`Service`](iceoryx2::service::Service)s that are accessible via the
/// [`Backend`](crate::traits::Backend) and announce local services
/// so they can be discovered remotely. Implementations query the
/// [`Backend`](crate::traits::Backend)'s communication layer to find active
/// [`Service`](iceoryx2::service::Service) and provide their [`StaticConfig`]s
/// to be processed by the caller.
///
/// # Examples
///
/// Using [`Discovery`] to list available [`Service`](iceoryx2::service::Service)s:
///
/// ```no_run
/// use iceoryx2_tunnel_backend::traits::Discovery;
///
/// fn list_services<DiscoveryError, ProcessingError>(discovery: &impl Discovery<DiscoveryError = DiscoveryError>) -> Result<(), DiscoveryError> {
///     discovery.discover::<ProcessingError>(&mut |static_config| {
///         println!("Found service: {:?}", static_config.name());
///         Ok(())
///     })?;
///     Ok(())
/// }
/// ```
///
/// Implementing a custom [`Discovery`]:
///
/// ```no_run
/// use iceoryx2_tunnel_backend::traits::Discovery;
/// use iceoryx2_tunnel_backend::types::discovery::ProcessDiscoveryFn;
/// use iceoryx2::service::static_config::StaticConfig;
///
/// struct MyDiscovery {
///     // Discovery state
/// }
///
/// #[derive(Debug)]
/// struct MyDiscoveryError;
/// impl core::fmt::Display for MyDiscoveryError {
///     fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
///         write!(f, "discovery failed")
///     }
/// }
/// impl core::error::Error for MyDiscoveryError {}
///
/// #[derive(Debug)]
/// struct MyAnnouncementError;
/// impl core::fmt::Display for MyAnnouncementError {
///     fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
///         write!(f, "discovery failed")
///     }
/// }
/// impl core::error::Error for MyAnnouncementError {}
///
/// impl Discovery for MyDiscovery {
///     type DiscoveryError = MyDiscoveryError;
///     type AnnouncementError = MyAnnouncementError;
///
///     fn announce(&self, static_config: &StaticConfig)
///         -> Result<(), Self::AnnouncementError> {
///         // Make the service described by the provided static config
///         // discoverable over the backend
///         Ok(())
///     }
///
///     fn discover<ProcessDiscoveryError>(
///         &self,
///         process_discovery: &mut ProcessDiscoveryFn<ProcessDiscoveryError>,
///     ) -> Result<(), Self::DiscoveryError> {
///         // Query backend for available services
///         // For each service found, call process_discovery with its
///         // static config
///         Ok(())
///     }
/// }
/// ```
pub trait Discovery {
    /// Error type that can occur during discovery operations.
    type DiscoveryError: Error;

    /// Error type that can occur during announcement operations.
    type AnnouncementError: Error;

    /// Announces a [`Service`](iceoryx2::service::Service) to make it
    /// discoverable by other hosts.
    ///
    /// This method broadcasts a [`Service`](iceoryx2::service::Service) available
    /// on the host over the [`Backend`](crate::traits::Backend)'s communication
    /// mechanism, making it available for discovery remotely.
    ///
    /// # Parameters
    ///
    /// * `static_config` - The [`StaticConfig`] of the service to announce.
    fn announce(&self, static_config: &StaticConfig) -> Result<(), Self::AnnouncementError>;

    /// Discovers available services and processes each one with the provided
    /// `process_discovery` callback.
    ///
    /// This method queries the backend's communication mechanism for all
    /// accessible [`Service`](iceoryx2::service::Service)s, then invokes
    /// `process_discovery` for each discovered [`StaticConfig`].
    /// Discovery continues until all services are processed or an error occurs.
    ///
    /// # Parameters
    ///
    /// * `process_discovery` - Callback provided by the caller to process the
    ///   [`StaticConfig`] of each discovered [`Service`](iceoryx2::service::Service).
    fn discover<E: Error, F: FnMut(&StaticConfig) -> Result<(), E>>(
        &self,
        process_discovery: F,
    ) -> Result<(), Self::DiscoveryError>;
}
