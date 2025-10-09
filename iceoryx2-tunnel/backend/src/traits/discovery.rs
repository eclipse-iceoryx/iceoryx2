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

use crate::types::discovery::ProcessDiscoveryFn;

/// Service discovery interface for finding services over the backend
/// communication mechanism.
///
/// `Discovery` enables enumeration and inspection of available iceoryx2
/// services that are accessible via the backend. Implementations
/// query the backend's communication layer to find active services and
/// provide their static configurations.
///
/// # Examples
///
/// Using discovery to list available services:
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
/// Implementing discovery for a custom backend:
///
/// ```no_run
/// use iceoryx2_tunnel_backend::traits::Discovery;
/// use iceoryx2_tunnel_backend::types::discovery::ProcessDiscoveryFn;
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
/// impl Discovery for MyDiscovery {
///     type DiscoveryError = MyDiscoveryError;
///
///     fn discover<ProcessDiscoveryError>(
///         &self,
///         process_discovery: &mut ProcessDiscoveryFn<ProcessDiscoveryError>,
///     ) -> Result<(), Self::DiscoveryError> {
///         // Query backend for available services
///         // For each service found, call process_discovery with its
///         // static_config
///         Ok(())
///     }
/// }
/// ```
///
/// # Errors
///
/// The `discover` method returns `DiscoveryError` when the backend
/// communication fails or when the backend fails to retrieve available
/// services. The provided `process_discovery` callback may also return errors,
/// which are propagated to the caller.
pub trait Discovery {
    /// Error type that can occur during discovery operations.
    type DiscoveryError: Error;

    /// Discovers available services and processes each one with the provided
    /// callback.
    ///
    /// This method queries the backend's communication mechanism for all
    /// accessible services, then invokes `process_discovery` for each
    /// service's static configuration. Discovery continues until all services
    /// are processed or an error occurs.
    ///
    /// # Parameters
    ///
    /// * `process_discovery` - Callback invoked for each discovered service,
    ///   receiving the service's `StaticConfig`.
    ///
    /// # Errors
    ///
    /// Returns `Err(DiscoveryError)` if unable to retrieve services via the
    /// Backend.
    fn discover<ProcessDiscoveryError>(
        &self,
        process_discovery: &mut ProcessDiscoveryFn<ProcessDiscoveryError>,
    ) -> Result<(), Self::DiscoveryError>;
}
