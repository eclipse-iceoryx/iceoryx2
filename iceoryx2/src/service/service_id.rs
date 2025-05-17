// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_container::semantic_string::*;
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_system_types::file_name::RestrictedFileName;
use iceoryx2_cal::hash::Hash;
use serde::{Deserialize, Serialize};

use super::{messaging_pattern::MessagingPattern, service_name::ServiceName};

const SERVICE_ID_CAPACITY: usize = 64;

/// The unique id of a [`Service`](crate::service::Service)
#[derive(Debug, Eq, PartialEq, Clone, Hash, ZeroCopySend, Serialize, Deserialize)]
#[repr(C)]
pub struct ServiceId(pub(crate) RestrictedFileName<SERVICE_ID_CAPACITY>);

impl ServiceId {
    pub(crate) fn new<Hasher: Hash>(
        service_name: &ServiceName,
        messaging_pattern: MessagingPattern,
    ) -> Self {
        let pattern_and_service = (messaging_pattern as u32).to_string() + service_name.as_str();
        let value = Hasher::new(pattern_and_service.as_bytes())
            .value()
            .as_base64url()
            .clone();

        Self(fatal_panic!(from "ServiceId::new()",
                   when RestrictedFileName::new(&value),
                   "This should never happen! The Hasher used to create the ServiceId created an illegal value ({value}, len = {}).", value.len()))
    }

    /// Returns the maximum string length of a [`ServiceId`]
    pub const fn max_number_of_characters() -> usize {
        SERVICE_ID_CAPACITY
    }

    /// Returns a str reference to the [`ServiceId`]
    pub fn as_str(&self) -> &str {
        // SAFETY: a SemanticString is always a valid UTF-8 string
        unsafe { core::str::from_utf8_unchecked(self.0.as_bytes()) }
    }
}
