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

use iceoryx2::{
    config::Config,
    service::{static_config::StaticConfig, Service},
};
use iceoryx2_bb_concurrency::cell::RefCell;
use iceoryx2_log::{fail, fatal_panic};
use iceoryx2_services_discovery::service_discovery::Tracker;
use iceoryx2_tunnel_backend::traits::Discovery;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DiscoveryError {
    TrackerSynchronization,
    DiscoveryProcessing,
}

impl core::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DiscoveryError::{self:?}")
    }
}

impl core::error::Error for DiscoveryError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum AnnouncementError {}

impl core::fmt::Display for AnnouncementError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "AnnouncementError::{self:?}")
    }
}

impl core::error::Error for AnnouncementError {}

#[derive(Debug)]
pub struct DiscoveryTracker<S: Service>(RefCell<Tracker<S>>);

impl<S: Service> DiscoveryTracker<S> {
    pub fn create(iceoryx_config: &Config) -> Self {
        let tracker = Tracker::new(iceoryx_config);
        DiscoveryTracker(RefCell::new(tracker))
    }
}

impl<S: Service> Discovery for DiscoveryTracker<S> {
    type DiscoveryError = DiscoveryError;
    type AnnouncementError = AnnouncementError;

    fn announce(
        &self,
        _static_config: &iceoryx2::service::static_config::StaticConfig,
    ) -> Result<(), Self::AnnouncementError> {
        // NOOP - iceoryx2 handles discovery internally
        Ok(())
    }

    fn discover<E: core::error::Error, F: FnMut(&StaticConfig) -> Result<(), E>>(
        &self,
        mut process_discovery: F,
    ) -> Result<(), Self::DiscoveryError> {
        let tracker = &mut self.0.borrow_mut();
        let (added, _removed) = fail!(
            from self,
            when tracker.sync(),
            with DiscoveryError::TrackerSynchronization,
            "Failed to synchronize tracker"
        );

        for id in added {
            match &tracker.get(&id) {
                Some(service_details) => {
                    fail!(
                        from self,
                        when process_discovery(&service_details.static_details),
                        with DiscoveryError::DiscoveryProcessing,
                        "Failed to process discovery event"
                    );
                }
                None => {
                    fatal_panic!(
                        from "DiscoveryTracker::discover",
                        "This should never happen. Service discovered by tracker is not retrievable."
                    )
                }
            }
        }

        Ok(())
    }
}
