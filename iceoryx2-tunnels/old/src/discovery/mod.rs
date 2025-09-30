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

use iceoryx2::service::static_config::StaticConfig as IceoryxServiceConfig;

mod iceoryx;
mod zenoh;

pub(crate) use iceoryx::*;
pub(crate) use zenoh::*;

/// Errors that can occur during service discovery operations.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DiscoveryError {
    /// Failed to create a service for discovery purposes.
    ServiceCreation,
    /// Failed to announce a service to make it discoverable.
    ServiceAnnouncement,
    /// Failed to create a port required for discovery communication.
    PortCreation,
    /// Failed to receive update information from a local port.
    UpdateFromLocalPort,
    /// Failed to receive update information from a remote port.
    UpdateFromRemotePort,
    /// Failed to receive update information from a discovery tracker.
    UpdateFromTracker,
}

impl core::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        core::write!(f, "DiscoveryError::{self:?}")
    }
}

impl core::error::Error for DiscoveryError {}

pub(crate) trait Discovery<ServiceType: iceoryx2::service::Service> {
    fn discover<OnDiscovered: FnMut(&IceoryxServiceConfig) -> Result<(), DiscoveryError>>(
        &mut self,
        on_discovered: &mut OnDiscovered,
    ) -> Result<(), DiscoveryError>;
}
