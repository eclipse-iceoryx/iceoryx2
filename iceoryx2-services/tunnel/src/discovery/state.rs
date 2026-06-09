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

use crate::tunnel::Origin;

/// The set of services offered by one side, keyed by hash.
#[derive(Debug, Default)]
pub(crate) struct OfferedServices {
    offered: BTreeMap<ServiceHash, StaticConfig>,
}

impl OfferedServices {
    fn insert(&mut self, static_config: StaticConfig) {
        self.offered
            .insert(*static_config.service_hash(), static_config);
    }

    /// Removes `hash`, returning its [`StaticConfig`] if it was offered.
    fn remove(&mut self, hash: &ServiceHash) -> Option<StaticConfig> {
        self.offered.remove(hash)
    }

    fn contains(&self, hash: &ServiceHash) -> bool {
        self.offered.contains_key(hash)
    }

    fn hashes(&self) -> impl Iterator<Item = &ServiceHash> {
        self.offered.keys()
    }

    fn iter(&self) -> impl Iterator<Item = (&ServiceHash, &StaticConfig)> {
        self.offered.iter()
    }
}

/// A borrowed, point-in-time view of all offered services.
pub(crate) struct Snapshot<'a> {
    local: &'a OfferedServices,
    remote: &'a OfferedServices,
}

impl<'a> Snapshot<'a> {
    /// All offered services with their config; a service offered by both sides
    /// appears once.
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

/// The services the tunnel has discovered, both locally and remotely.
#[derive(Debug, Default)]
pub(crate) struct DiscoveryState {
    local: OfferedServices,
    remote: OfferedServices,
}

impl DiscoveryState {
    /// Records `static_config` as offered from `origin`.
    pub(crate) fn set_offered(&mut self, origin: Origin, static_config: StaticConfig) {
        match origin {
            Origin::Local => self.local.insert(static_config),
            Origin::Remote => self.remote.insert(static_config),
        }
    }

    /// Marks `hash` as no longer offered from `origin`, returning its
    /// [`StaticConfig`] if that side was offering it.
    pub(crate) fn set_not_offered(
        &mut self,
        origin: Origin,
        hash: &ServiceHash,
    ) -> Option<StaticConfig> {
        match origin {
            Origin::Local => self.local.remove(hash),
            Origin::Remote => self.remote.remove(hash),
        }
    }

    /// Whether `origin` currently offers `hash`.
    pub(crate) fn is_offered_by(&self, origin: Origin, hash: &ServiceHash) -> bool {
        match origin {
            Origin::Local => self.local.contains(hash),
            Origin::Remote => self.remote.contains(hash),
        }
    }

    /// A view over all services offered by either side.
    pub(crate) fn snapshot(&self) -> Snapshot<'_> {
        Snapshot {
            local: &self.local,
            remote: &self.remote,
        }
    }

    /// Hashes currently offered locally.
    pub(crate) fn locally_offered(&self) -> impl Iterator<Item = &ServiceHash> {
        self.local.hashes()
    }

    /// Hashes currently offered remotely.
    #[allow(dead_code)] // symmetry with `locally_offered`;
    pub(crate) fn remotely_offered(&self) -> impl Iterator<Item = &ServiceHash> {
        self.remote.hashes()
    }
}
