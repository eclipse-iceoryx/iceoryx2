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

use crate::types::identity::GatewayId;
use crate::types::service_description::ServiceDescription;

/// A change to the set of services offered by one remote gateway.
///
/// Several gateways may offer the same service. Each reports its own
/// additions and removals.
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum DiscoveryUpdate {
    /// The gateway started offering a service.
    Added(GatewayId, ServiceDescription),
    /// The gateway stopped offering a service.
    Removed(GatewayId, ServiceHash),
}

/// A change to the set of services offered by the local gateway, to be
/// announced over the backend.
#[derive(Debug, Clone, Copy)]
pub enum Announcement<'a> {
    /// A service became available.
    Added(&'a ServiceDescription),
    /// A service disappeared.
    Removed(&'a ServiceHash),
}
