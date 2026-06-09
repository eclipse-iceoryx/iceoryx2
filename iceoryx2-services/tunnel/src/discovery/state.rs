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

use alloc::collections::BTreeMap;

use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2::service::static_config::StaticConfig;

/// Side of the system that a discovery event refers to.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum Origin {
    /// Service was discovered (or its absence detected) on the local iceoryx system.
    Local,
    /// Service was discovered (or its absence detected) over the backend.
    Remote,
}

#[derive(Debug)]
struct OfferedService {
    static_config: StaticConfig,
    last_seen: u64,
}

/// The set of services offered by one side, keyed by hash.
#[derive(Debug, Default)]
pub(crate) struct OfferedServices {
    offered: BTreeMap<ServiceHash, OfferedService>,
    epoch: u64,
}

impl OfferedServices {
    /// Advances and returns the epoch, beginning a new update session.
    fn next_epoch(&mut self) -> u64 {
        self.epoch = self.epoch.wrapping_add(1);
        self.epoch
    }

    /// Records `config` as offered, stamping it with `epoch`. The caller passes
    /// its session epoch so all updates in a session share one stamp.
    fn insert(&mut self, config: StaticConfig, epoch: u64) {
        self.offered.insert(
            *config.service_hash(),
            OfferedService {
                static_config: config,
                last_seen: epoch,
            },
        );
    }

    /// Removes `hash`, returning its [`StaticConfig`] if it was offered.
    fn remove(&mut self, hash: &ServiceHash) -> Option<StaticConfig> {
        self.offered.remove(hash).map(|o| o.static_config)
    }

    fn contains(&self, hash: &ServiceHash) -> bool {
        self.offered.contains_key(hash)
    }

    fn iter(&self) -> impl Iterator<Item = (&ServiceHash, &StaticConfig)> {
        self.offered
            .iter()
            .map(|(hash, offered_service)| (hash, &offered_service.static_config))
    }

    /// Reconciles the offered services, taking an external collection as the
    /// target.
    fn reconcile<'c, E>(
        &mut self,
        target: impl Iterator<Item = &'c StaticConfig>,
        mut on_added: impl FnMut(&StaticConfig) -> Result<(), E>,
        mut on_removed: impl FnMut(&StaticConfig) -> Result<(), E>,
    ) -> Result<(), E> {
        let epoch = self.next_epoch();

        for static_config in target {
            let hash = *static_config.service_hash();
            if let Some(offered_service) = self.offered.get_mut(&hash) {
                offered_service.last_seen = epoch;
            } else {
                on_added(static_config)?;
                self.insert(static_config.clone(), epoch);
            }
        }

        let mut result = Ok(());
        self.offered.retain(|_, offered_service| {
            if offered_service.last_seen == epoch {
                true
            } else {
                if result.is_ok() {
                    result = on_removed(&offered_service.static_config);
                }
                false
            }
        });
        result
    }
}

/// A borrowed, point-in-time view of all offered services.
pub(crate) struct Snapshot<'a> {
    local: &'a OfferedServices,
    remote: &'a OfferedServices,
}

impl<'a> Snapshot<'a> {
    /// All offered services. A service offered by both sides
    /// appears only once.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&'a ServiceHash, &'a StaticConfig)> {
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

/// Handle for applying incremental (delta) updates to one side's offered
/// services within a single epoch.
pub(crate) struct DeltaUpdate<'a> {
    offered_services: &'a mut OfferedServices,
    origin: Origin,
    epoch: u64,
}

impl DeltaUpdate<'_> {
    /// The side these updates apply to.
    pub(crate) fn origin(&self) -> Origin {
        self.origin
    }

    /// Whether the side currently offers `hash`.
    pub(crate) fn is_offered(&self, hash: &ServiceHash) -> bool {
        self.offered_services.contains(hash)
    }

    /// Records `static_config` as offered, stamped with this handle's epoch.
    pub(crate) fn set_offered(&mut self, static_config: StaticConfig) {
        self.offered_services.insert(static_config, self.epoch);
    }

    /// Marks `hash` as no longer offered, returning its [`StaticConfig`] if it
    /// was offered.
    pub(crate) fn set_not_offered(&mut self, hash: &ServiceHash) -> Option<StaticConfig> {
        self.offered_services.remove(hash)
    }
}

/// The services the tunnel has discovered, both locally and remotely.
#[derive(Debug, Default)]
pub(crate) struct DiscoveryState {
    local: OfferedServices,
    remote: OfferedServices,
}

impl DiscoveryState {
    /// Returns a handle for applying delta updates to `origin`'s offered
    /// services. Advances that side's epoch, beginning a new update session.
    pub(crate) fn delta_update(&mut self, origin: Origin) -> DeltaUpdate<'_> {
        let offered_services = match origin {
            Origin::Local => &mut self.local,
            Origin::Remote => &mut self.remote,
        };
        let epoch = offered_services.next_epoch();

        DeltaUpdate {
            offered_services,
            origin,
            epoch,
        }
    }

    /// Forces `origin`'s offerings to match an external target set, calling
    /// the provided callbacks on addition or removal.
    pub(crate) fn force_update<'c, E>(
        &mut self,
        origin: Origin,
        target: impl Iterator<Item = &'c StaticConfig>,
        on_added: impl FnMut(&StaticConfig) -> Result<(), E>,
        on_removed: impl FnMut(&StaticConfig) -> Result<(), E>,
    ) -> Result<(), E> {
        match origin {
            Origin::Local => self.local.reconcile(target, on_added, on_removed),
            Origin::Remote => self.remote.reconcile(target, on_added, on_removed),
        }
    }

    /// A view over all services offered by either side.
    pub(crate) fn snapshot(&self) -> Snapshot<'_> {
        Snapshot {
            local: &self.local,
            remote: &self.remote,
        }
    }
}
