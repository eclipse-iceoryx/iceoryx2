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

use iceoryx2::service::Service;
use iceoryx2_services_discovery::service_discovery::{SyncError, Tracker};

use iceoryx2_tunnels_traits::Discovery;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DiscoveryError {
    FailedToSynchronizeTracker,
}

impl From<SyncError> for DiscoveryError {
    fn from(_: SyncError) -> Self {
        DiscoveryError::FailedToSynchronizeTracker
    }
}

pub struct DiscoveryTracker<S: Service>(Tracker<S>);

impl<S: Service> DiscoveryTracker<S> {
    pub fn new(iceoryx_config: &iceoryx2::config::Config) -> Self {
        let tracker = Tracker::new(iceoryx_config);
        DiscoveryTracker(tracker)
    }
}

impl<S: Service> Discovery<S> for DiscoveryTracker<S> {
    type Handle = Self;
    type DiscoveryError = DiscoveryError;

    fn discover<
        F: FnMut(&iceoryx2::service::static_config::StaticConfig) -> Result<(), Self::DiscoveryError>,
    >(
        handle: &mut Self::Handle,
        process_discovery: &mut F,
    ) -> Result<(), Self::DiscoveryError> {
        let tracker = &mut handle.0;
        let (added, _removed) = tracker.sync().unwrap();

        for id in added {
            process_discovery(&tracker.get(&id).unwrap().static_details)?;
        }

        Ok(())
    }
}
