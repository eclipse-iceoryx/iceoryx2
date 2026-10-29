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

use r2r_rcl::rmw_service_info_t;

use crate::rcl::RequestId;

/// Per-message metadata accompanying a taken request or response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServiceInfo {
    pub request_id: RequestId,
    pub source_timestamp_ns: i64,
    pub received_timestamp_ns: i64,
}

impl From<&rmw_service_info_t> for ServiceInfo {
    fn from(info: &rmw_service_info_t) -> Self {
        Self {
            request_id: RequestId::from(&info.request_id),
            source_timestamp_ns: info.source_timestamp,
            received_timestamp_ns: info.received_timestamp,
        }
    }
}
