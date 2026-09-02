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

//! Identity of a service description on the zenoh wire.

use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_gateway_backend::types::service_description::ServiceDescription;

use crate::wire::description::EncodedDescription;
use crate::wire::fingerprint::Fingerprint;

/// Identifies a specific service instantiation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ServiceDescriptor {
    pub service_hash: ServiceHash,
    pub fingerprint: Fingerprint,
}

impl ServiceDescriptor {
    pub fn new(service_hash: ServiceHash, fingerprint: Fingerprint) -> Self {
        Self {
            service_hash,
            fingerprint,
        }
    }
}

/// Derives the descriptor of a description by encoding it.
pub fn describe(description: &ServiceDescription) -> Result<ServiceDescriptor, postcard::Error> {
    let encoded = EncodedDescription::encode(description)?;
    Ok(ServiceDescriptor::new(
        description.service_hash,
        encoded.fingerprint(),
    ))
}
