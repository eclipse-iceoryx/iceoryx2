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
    prelude::CallbackProgression,
    service::{service_id::ServiceId, Service, ServiceDetails, ServiceListError},
};
use std::collections::{HashMap, HashSet};

/// Errors that can occur during service synchronization.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SyncError {
    /// The caller does not have permissions to look up services.
    InsufficientPermissions,

    /// Failure looking up services present in the iceoryx2 system.
    ServiceLookupFailure,
}

impl core::fmt::Display for SyncError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
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

/// A tracker for monitoring services of a specific type.
///
/// The `Tracker` keeps track of services in the system, allowing for discovery
/// of new services and detection of services that are no longer available.
///
/// # Type Parameters
///
/// * `S` - The type of service to track, which must implement the `Service` trait
#[derive(Debug)]
pub struct Tracker<S: Service> {
    services: HashMap<ServiceId, ServiceDetails<S>>,
}

impl<S: Service> Default for Tracker<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Service> Tracker<S> {
    /// Create a new Monitor instance.
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    /// Synchronizes the tracker with the current state of services in the system.
    ///
    /// This method queries the system for all services of type `S` and updates the tracker's
    /// internal state. It identifies new services that have appeared since the last sync
    /// and services that are no longer available.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration used to discover services
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * A vector of service IDs that were newly discovered, details are stored in the tracker and
    ///   retrievable with `Tracker::get()`
    /// * A vector of service details for services that are no longer available, these details are
    ///   no longer stored in the tracker
    pub fn sync(
        &mut self,
        config: &Config,
    ) -> Result<(Vec<ServiceId>, Vec<ServiceDetails<S>>), SyncError> {
        let mut discovered_ids = HashSet::<ServiceId>::new();
        let mut added_ids = Vec::<ServiceId>::new();

        S::list(config, |service| {
            let id = service.static_details.service_id().clone();
            discovered_ids.insert(id.clone());

            // Track new services.
            if !self.services.contains_key(&id) {
                self.services.insert(id.clone(), service);
                added_ids.push(id);
            }
            CallbackProgression::Continue
        })?;

        // Get the details of the services not discovered
        let mut removed_services = Vec::new();
        let undiscovered_ids: Vec<ServiceId> = self
            .services
            .keys()
            .filter(|&id| !discovered_ids.contains(id))
            .cloned()
            .collect();

        for id in undiscovered_ids {
            if let Some(service) = self.services.remove(&id) {
                removed_services.push(service);
            }
        }

        Ok((added_ids, removed_services))
    }

    /// Retrieves service details for a specific service ID if tracked.
    ///
    /// # Arguments
    ///
    /// * `id` - The service ID to look up
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the service details if tracked, or `None` if not tracked
    pub fn get(&self, id: &ServiceId) -> Option<&ServiceDetails<S>> {
        self.services.get(id)
    }

    /// Retrieves service details all the services that are being currently tracked.
    ///
    /// # Returns
    ///
    /// An `Vec` containing a reference to the service details all the services that are being tracked
    pub fn get_all(&self) -> Vec<&ServiceDetails<S>> {
        self.services.values().collect()
    }
}
