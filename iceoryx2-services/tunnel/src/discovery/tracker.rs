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

use alloc::vec::Vec;

use iceoryx2::config::Config;
use iceoryx2::service::Service;
use iceoryx2::service::ServiceDetails;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2::service::static_config::StaticConfig;
use iceoryx2_bb_concurrency::cell::RefCell;
use iceoryx2_log::{fail, fatal_panic};
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

/// Result of a single [`DiscoveryTracker::sync`] cycle.
#[derive(Debug)]
pub struct DiscoverySync {
    /// Static configs of services newly visible in the local registry.
    pub added: Vec<StaticConfig>,
    /// Hashes of services that have disappeared from the local registry entirely.
    pub removed: Vec<ServiceHash>,
}

#[derive(Debug)]
pub struct DiscoveryTracker<S: Service> {
    tracker: RefCell<Tracker<S>>,
}

impl<S: Service> DiscoveryTracker<S> {
    pub fn create(iceoryx_config: &Config) -> Self {
        DiscoveryTracker {
            tracker: RefCell::new(Tracker::new(iceoryx_config)),
        }
    }

    /// Synchronises against the local iceoryx registry and returns the services
    /// that newly appeared or fully disappeared since the last sync.
    pub fn sync(&self) -> Result<DiscoverySync, DiscoveryError> {
        let origin = "DiscoveryTracker::sync";
        let mut tracker = self.tracker.borrow_mut();
        let (added_ids, removed_services) = fail!(
            from origin,
            when tracker.sync(),
            with DiscoveryError::TrackerSynchronization,
            "Failed to synchronize tracker"
        );

        let added = added_ids
            .into_iter()
            .map(|id| match tracker.get(&id) {
                Some(details) => details.static_details.clone(),
                None => fatal_panic!(
                    from origin,
                    "This should never happen. Service discovered by tracker is not retrievable."
                ),
            })
            .collect();
        let removed = removed_services
            .into_iter()
            .map(|d| *d.static_details.service_hash())
            .collect();

        Ok(DiscoverySync { added, removed })
    }

    /// Drops the cached snapshot for `hash`, allowing a subsequent [`sync`]
    /// to re-emit `added` if the service is still present in the registry.
    /// Used by the tunnel after tearing down its ports and relay so a same-hash
    /// service that is recreated by a user is observed as a fresh addition.
    ///
    /// [`sync`]: Self::sync
    pub fn forget(&self, hash: &ServiceHash) {
        let mut tracker = self.tracker.borrow_mut();
        tracker.forget(hash);
    }

    /// Invokes `f` with the cached snapshot for `hash`, or `None` if the
    /// service is not currently tracked.
    pub fn get<R>(&self, hash: &ServiceHash, f: impl FnOnce(Option<&ServiceDetails<S>>) -> R) -> R {
        let tracker = self.tracker.borrow();
        f(tracker.get(hash))
    }
}
