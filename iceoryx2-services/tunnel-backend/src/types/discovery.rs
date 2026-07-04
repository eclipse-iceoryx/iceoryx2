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

use iceoryx2::service::service_hash::ServiceHash;

use crate::types::service_description::ServiceDescription;

/// A change to the set of services offered on one side of the tunnel.
///
/// Owning variant.
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum DiscoveryUpdate {
    /// A service became available; carries its description.
    Added(ServiceDescription),
    /// A service disappeared; carries its identity.
    Removed(ServiceHash),
}

/// A change to the set of services offered on one side of the tunnel.
///
/// Non-owning variant.
#[derive(Debug, Clone, Copy)]
pub enum DiscoveryUpdateRef<'a> {
    /// A service became available; carries its description.
    Added(&'a ServiceDescription),
    /// A service disappeared; carries its identity.
    Removed(&'a ServiceHash),
}

impl<'a> From<&'a DiscoveryUpdate> for DiscoveryUpdateRef<'a> {
    fn from(update: &'a DiscoveryUpdate) -> Self {
        match update {
            DiscoveryUpdate::Added(description) => DiscoveryUpdateRef::Added(description),
            DiscoveryUpdate::Removed(hash) => DiscoveryUpdateRef::Removed(hash),
        }
    }
}
