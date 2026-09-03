// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use alloc::collections::{BTreeMap, BTreeSet};

use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_gateway_backend::types::identity::GatewayId;
use iceoryx2_gateway_backend::types::service_description::ServiceDescription;

// A service offered locally.
#[derive(Debug)]
struct LocalService {
    description: ServiceDescription,
    last_seen: u64,
}

/// The set of services offered locally.
#[derive(Debug, Default)]
pub(crate) struct LocalServices {
    offered: BTreeMap<ServiceHash, LocalService>,
    epoch: u64,
}

impl LocalServices {
    /// Advances and returns the epoch, beginning a new update session.
    fn next_epoch(&mut self) -> u64 {
        self.epoch = self.epoch.wrapping_add(1);
        self.epoch
    }

    /// Records `description` as offered, stamping it with `epoch`. The caller
    /// passes its session epoch so all updates in a session share one stamp.
    fn insert(&mut self, description: ServiceDescription, epoch: u64) {
        self.offered.insert(
            description.service_hash,
            LocalService {
                description,
                last_seen: epoch,
            },
        );
    }

    /// Removes `hash`, returning its [`ServiceDescription`] if it was offered.
    fn remove(&mut self, hash: &ServiceHash) -> Option<ServiceDescription> {
        self.offered.remove(hash).map(|o| o.description)
    }

    /// Whether `hash` is offered locally.
    pub(crate) fn contains(&self, hash: &ServiceHash) -> bool {
        self.offered.contains_key(hash)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&ServiceHash, &ServiceDescription)> {
        self.offered
            .iter()
            .map(|(hash, service)| (hash, &service.description))
    }

    /// Returns a handle for applying delta updates. Advances the epoch,
    /// beginning a new update session.
    pub(crate) fn delta_update(&mut self) -> DeltaUpdate<'_> {
        let epoch = self.next_epoch();
        DeltaUpdate { local: self, epoch }
    }

    /// Forces the offered services to match an external target set, calling
    /// the provided callbacks on addition or removal.
    pub(crate) fn force_update<E>(
        &mut self,
        target: impl Iterator<Item = ServiceDescription>,
        mut on_added: impl FnMut(&ServiceDescription) -> Result<(), E>,
        mut on_removed: impl FnMut(&ServiceDescription) -> Result<(), E>,
    ) -> Result<(), E> {
        let epoch = self.next_epoch();

        for description in target {
            let hash = description.service_hash;
            if let Some(service) = self.offered.get_mut(&hash) {
                service.last_seen = epoch;
            } else {
                on_added(&description)?;
                self.insert(description, epoch);
            }
        }

        let mut result = Ok(());
        self.offered.retain(|_, service| {
            if service.last_seen == epoch {
                true
            } else {
                if result.is_ok() {
                    result = on_removed(&service.description);
                }
                false
            }
        });
        result
    }
}

/// A remotely offered service grouped together with the remote gateways
/// that offer it.
#[derive(Debug)]
struct RemoteService {
    description: ServiceDescription,
    gateways: BTreeSet<GatewayId>,
}

/// The set of services offered over the backend.
#[derive(Debug, Default)]
pub(crate) struct RemoteServices {
    offered: BTreeMap<ServiceHash, RemoteService>,
}

impl RemoteServices {
    /// Records `gateway` as offering `description`. The description of the
    /// first announcing gateway is kept.
    pub(crate) fn add(&mut self, gateway: GatewayId, description: ServiceDescription) {
        let hash = description.service_hash;
        self.offered
            .entry(hash)
            .or_insert_with(|| RemoteService {
                description,
                gateways: BTreeSet::new(),
            })
            .gateways
            .insert(gateway);
    }

    /// Removes `gateway` from the gateways offering `hash`. Returns the
    /// [`ServiceDescription`] once no gateway offers the service anymore.
    pub(crate) fn remove(
        &mut self,
        gateway: &GatewayId,
        hash: &ServiceHash,
    ) -> Option<ServiceDescription> {
        let service = self.offered.get_mut(hash)?;
        service.gateways.remove(gateway);
        if !service.gateways.is_empty() {
            return None;
        }
        self.offered.remove(hash).map(|service| service.description)
    }

    fn contains(&self, hash: &ServiceHash) -> bool {
        self.offered.contains_key(hash)
    }

    fn iter(&self) -> impl Iterator<Item = (&ServiceHash, &ServiceDescription)> {
        self.offered
            .iter()
            .map(|(hash, service)| (hash, &service.description))
    }
}

/// A borrowed, point-in-time view of all offered services.
pub(crate) struct Snapshot<'a> {
    local: &'a LocalServices,
    remote: &'a RemoteServices,
}

impl<'a> Snapshot<'a> {
    /// All offered services. A service offered by both sides
    /// appears only once.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&'a ServiceHash, &'a ServiceDescription)> {
        let local = self.local;
        local.iter().chain(
            self.remote
                .iter()
                .filter(move |&(hash, _)| !local.contains(hash)),
        )
    }

    /// Whether `hash` is offered by either side.
    pub(crate) fn contains(&self, hash: &ServiceHash) -> bool {
        self.local.contains(hash) || self.remote.contains(hash)
    }
}

/// Handle for applying incremental (delta) updates to the locally offered
/// services within a single epoch.
pub(crate) struct DeltaUpdate<'a> {
    local: &'a mut LocalServices,
    epoch: u64,
}

impl DeltaUpdate<'_> {
    /// Records `description` as offered, stamped with this handle's epoch.
    pub(crate) fn set_offered(&mut self, description: ServiceDescription) {
        self.local.insert(description, self.epoch);
    }

    /// Marks `hash` as no longer offered, returning its [`ServiceDescription`]
    /// if it was offered.
    pub(crate) fn set_not_offered(&mut self, hash: &ServiceHash) -> Option<ServiceDescription> {
        self.local.remove(hash)
    }
}

/// The services the gateway has discovered, both locally and remotely.
#[derive(Debug, Default)]
pub(crate) struct DiscoveryState {
    local: LocalServices,
    remote: RemoteServices,
}

impl DiscoveryState {
    pub(crate) fn local(&self) -> &LocalServices {
        &self.local
    }

    pub(crate) fn local_mut(&mut self) -> &mut LocalServices {
        &mut self.local
    }

    pub(crate) fn remote_mut(&mut self) -> &mut RemoteServices {
        &mut self.remote
    }

    /// A view over all services offered by either side.
    pub(crate) fn snapshot(&self) -> Snapshot<'_> {
        Snapshot {
            local: &self.local,
            remote: &self.remote,
        }
    }
}
