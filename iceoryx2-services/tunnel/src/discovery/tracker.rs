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

use iceoryx2::config::Config;
use iceoryx2::service::Service;
use iceoryx2::service::ServiceDetails;
use iceoryx2_log::fail;
use iceoryx2_services_discovery::service_discovery::Tracker;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DiscoveryError {
    TrackerSynchronization,
}

impl core::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DiscoveryError::{self:?}")
    }
}

impl core::error::Error for DiscoveryError {}

#[derive(Debug)]
pub struct DiscoveryTracker<S: Service> {
    tracker: Tracker<S>,
}

impl<S: Service> DiscoveryTracker<S> {
    pub fn create(iceoryx_config: &Config) -> Self {
        DiscoveryTracker {
            tracker: Tracker::new(iceoryx_config),
        }
    }

    /// Synchronize the tracker with the services currently in the system.
    pub fn sync(&mut self) -> Result<(), DiscoveryError> {
        let origin = "DiscoveryTracker::sync";
        fail!(
            from origin,
            when self.tracker.sync(|_| {}),
            with DiscoveryError::TrackerSynchronization,
            "Failed to synchronize tracker"
        );
        Ok(())
    }

    /// Iterates over all currently-tracked services.
    pub fn iter(&self) -> impl Iterator<Item = &ServiceDetails<S>> {
        self.tracker.iter()
    }
}
