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
use iceoryx2_link_backend::service_description::ServiceDescriptor;

/// A change to this tunnel's offers.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)] // a descriptor is announced by value, announcements are rare and short-lived
pub enum Announcement {
    Offered { descriptor: ServiceDescriptor },
    Withdrawn { hash: ServiceHash },
}
