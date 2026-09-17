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
use r2r_rcl::{RMW_GID_STORAGE_SIZE, rmw_gid_t};

/// Wraps the RMW identifier of an endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gid([u8; RMW_GID_STORAGE_SIZE as usize]);

impl Gid {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<&rmw_gid_t> for Gid {
    fn from(gid: &rmw_gid_t) -> Self {
        Self(gid.data)
    }
}
