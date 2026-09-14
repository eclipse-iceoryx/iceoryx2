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
use iceoryx2::service::service_name::ServiceName;
use serde::{Deserialize, Serialize};

use crate::service_description::{ServiceDescription, ServiceTypes};

/// What identifies a service across the boundary.
///
/// Services with the same descriptor can exchange data, whatever
/// their settings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ServiceDescriptor {
    pub name: ServiceName,
    pub hash: ServiceHash,
    pub types: ServiceTypes,
}

impl From<&ServiceDescription> for ServiceDescriptor {
    fn from(description: &ServiceDescription) -> Self {
        Self {
            name: description.name(),
            hash: description.hash,
            types: description.types.clone(),
        }
    }
}
