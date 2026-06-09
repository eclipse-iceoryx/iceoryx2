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

/// The set of services currently offered by one side, keyed by hash.
#[derive(Debug, Default)]
pub(crate) struct DiscoverySnapshot {
    offered: BTreeMap<ServiceHash, StaticConfig>,
}

impl DiscoverySnapshot {
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
}

/// The services the tunnel has discovered, both locally and remotely.
#[derive(Debug, Default)]
pub(crate) struct DiscoveryState {
    local: DiscoverySnapshot,
    remote: DiscoverySnapshot,
}

impl DiscoveryState {
    fn snapshot(&self, origin: Origin) -> &DiscoverySnapshot {
        match origin {
            Origin::Local => &self.local,
            Origin::Remote => &self.remote,
        }
    }

    fn snapshot_mut(&mut self, origin: Origin) -> &mut DiscoverySnapshot {
        match origin {
            Origin::Local => &mut self.local,
            Origin::Remote => &mut self.remote,
        }
    }

    /// Records `static_config` as offered from `origin`.
    pub(crate) fn set_offered(&mut self, origin: Origin, static_config: StaticConfig) {
        self.snapshot_mut(origin).insert(static_config);
    }

    /// Marks `hash` as no longer offered from `origin`, returning its
    /// [`StaticConfig`] if that side was offering it.
    pub(crate) fn set_not_offered(
        &mut self,
        origin: Origin,
        hash: &ServiceHash,
    ) -> Option<StaticConfig> {
        self.snapshot_mut(origin).remove(hash)
    }

    /// Whether `origin` currently offers `hash`.
    pub(crate) fn is_offered_by(&self, origin: Origin, hash: &ServiceHash) -> bool {
        self.snapshot(origin).contains(hash)
    }

    /// Whether any side currently offers `hash`.
    pub(crate) fn is_offered(&self, hash: &ServiceHash) -> bool {
        self.local.contains(hash) || self.remote.contains(hash)
    }

    /// Hashes currently offered locally.
    pub(crate) fn locally_offered(&self) -> impl Iterator<Item = &ServiceHash> {
        self.local.hashes()
    }

    /// Hashes currently offered remotely.
    #[allow(dead_code)] // symmetry with `locally_offered`; wired in by reconciliation
    pub(crate) fn remotely_offered(&self) -> impl Iterator<Item = &ServiceHash> {
        self.remote.hashes()
    }
}
