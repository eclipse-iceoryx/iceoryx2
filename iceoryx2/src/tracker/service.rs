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

use crate::config;
use crate::service::service_id::ServiceId;
use crate::service::static_config::StaticConfig;
use crate::service::{Service, ServiceDetails};
use iceoryx2_bb_elementary::CallbackProgression;
use std::collections::{HashMap, HashSet};

/// A tracker for monitoring services of a specific type.
///
/// The `Tracker` keeps track of services in the system, allowing for discovery
/// of new services and detection of services that are no longer available.
///
/// # Type Parameters
///
/// * `S` - The type of service to track, which must implement the `Service` trait
pub struct Tracker<S: Service> {
    services: HashMap<ServiceId, ServiceDetails<S>>,
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
    /// This method discovers all services of type `S` in the provided configuration,
    /// tracks new services, and removes services that are no longer available.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to use for service discovery
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * A set of service IDs of the newly discovered services, whose details are stored
    /// * A vector of static configurations for services, whose details are no longer stored
    pub fn sync(&mut self, config: &config::Config) -> (HashSet<ServiceId>, Vec<StaticConfig>) {
        let mut discovered_ids = HashSet::<ServiceId>::new();
        let mut new_ids = HashSet::<ServiceId>::new();

        let _ = S::list(config, |service| {
            let id = service.static_details.service_id();
            discovered_ids.insert(id.clone());

            // Track new services.
            if !self.services.contains_key(id) {
                new_ids.insert(id.clone());
                self.services.insert(id.clone(), service);
            }
            CallbackProgression::Continue
        });

        // Get the details of the services not discovered
        let undiscovered_ids: HashSet<ServiceId> = self
            .services
            .keys()
            .filter(|&id| !discovered_ids.contains(id))
            .cloned()
            .collect();
        let undiscovered_details = self.get_many(&undiscovered_ids);

        // Remove the services that were not discovered
        self.services.retain(|id, _| discovered_ids.contains(id));

        (new_ids, undiscovered_details)
    }

    /// Returns the static details of a single service with the given ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The service ID to retrieve details for
    ///
    /// # Returns
    ///
    /// An option containing the static details if the service was found, or None otherwise
    pub fn get_one(&self, id: &ServiceId) -> Option<StaticConfig> {
        self.services
            .get(id)
            .map(|service| service.static_details.clone())
    }

    /// Returns the static details of the services with the given IDs.
    ///
    /// # Arguments
    ///
    /// * `ids` - A set of service IDs to retrieve details for
    ///
    /// # Returns
    ///
    /// A vector of static configurations for the services that were found
    pub fn get_many<'a, I>(&self, ids: I) -> Vec<StaticConfig>
    where
        I: IntoIterator<Item = &'a ServiceId>,
    {
        ids.into_iter()
            .filter_map(|id| {
                self.services
                    .get(id)
                    .map(|service| service.static_details.clone())
            })
            .collect()
    }
}
