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

use alloc::collections::BTreeMap;
use alloc::collections::btree_map::Entry;

use iceoryx2::{
    config::Config,
    prelude::CallbackProgression,
    service::{Service, ServiceDetails, ServiceListError, service_hash::ServiceHash},
};

/// Errors that can occur during service synchronization.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SyncError {
    /// The caller does not have permissions to look up services.
    InsufficientPermissions,

    /// Failure looking up services present in the iceoryx2 system.
    ServiceLookupFailure,
}

impl core::fmt::Display for SyncError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SyncError::{self:?}")
    }
}

impl core::error::Error for SyncError {}

impl From<ServiceListError> for SyncError {
    fn from(error: ServiceListError) -> Self {
        match error {
            ServiceListError::InsufficientPermissions => Self::InsufficientPermissions,
            ServiceListError::InternalError => Self::ServiceLookupFailure,
        }
    }
}

/// Event emitted by [`Tracker::sync`] for each detected service change.
#[derive(Debug)]
pub enum TrackerEvent<'a, S: Service> {
    /// A service newly visible to the tracker.
    Added(&'a ServiceDetails<S>),
    /// A service no longer visible to the tracker.
    Removed(&'a ServiceDetails<S>),
}

#[derive(Debug)]
struct TrackedEntry<S: Service> {
    details: ServiceDetails<S>,
    /// Epoch in which this entry was last observed in the system listing.
    seen: u64,
}

/// A tracker for monitoring services of a specific type.
///
/// The `Tracker` keeps track of services in the system, allowing for discovery
/// of new services and detection of services that are no longer available.
///
/// # Type Parameters
///
/// * `S` - The type of service to track, which must implement the `Service` trait
#[derive(Debug, Default)]
pub struct Tracker<S: Service> {
    config: Config,
    services: BTreeMap<ServiceHash, TrackedEntry<S>>,
    /// Monotonic counter incremented every [`Tracker::sync`].
    /// Used to detect tracked services that become stale.
    epoch: u64,
}

impl<S: Service> Tracker<S> {
    /// Creates a new tracker.
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
            services: BTreeMap::new(),
            epoch: 0,
        }
    }

    /// Synchronises the tracker with the current state of services in the
    /// system.
    pub fn sync<F>(&mut self, mut on_event: F) -> Result<(), SyncError>
    where
        F: for<'a> FnMut(TrackerEvent<'a, S>),
    {
        // Bump the epoch; entries refreshed this cycle will carry the new
        // value, anything older is then known to have disappeared.
        self.epoch = self.epoch.wrapping_add(1);
        let epoch = self.epoch;

        S::list(&self.config, |service| {
            let id = *service.static_details.service_hash();

            match self.services.entry(id) {
                Entry::Vacant(slot) => {
                    let entry = slot.insert(TrackedEntry {
                        details: service,
                        seen: epoch,
                    });
                    on_event(TrackerEvent::Added(&entry.details));
                }
                Entry::Occupied(mut slot) => {
                    let entry = slot.get_mut();
                    entry.details = service;
                    entry.seen = epoch;
                }
            }

            CallbackProgression::Continue
        })?;

        self.services.retain(|_, entry| {
            if entry.seen == epoch {
                true
            } else {
                on_event(TrackerEvent::Removed(&entry.details));
                false
            }
        });

        Ok(())
    }

    /// Removes a service from the tracker without waiting for it to
    /// disappear from the system listing. Intended for callers that have
    /// determined a service should logically be considered removed
    /// (e.g., the service is only held by the tunnel, which should not
    /// contribute to the discovery state).
    pub fn forget(&mut self, id: &ServiceHash) -> Option<ServiceDetails<S>> {
        self.services.remove(id).map(|e| e.details)
    }

    /// Retrieves service details for a specific tracked service.
    pub fn get(&self, id: &ServiceHash) -> Option<&ServiceDetails<S>> {
        self.services.get(id).map(|e| &e.details)
    }

    /// Iterates over all currently-tracked services.
    pub fn iter(&self) -> impl Iterator<Item = &ServiceDetails<S>> + '_ {
        self.services.values().map(|e| &e.details)
    }
}
