// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

#![allow(non_camel_case_types)]

use crate::iox2_type_detail_t;
use iceoryx2::service::static_config::blackboard::StaticConfig;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct iox2_static_config_blackboard_t {
    pub max_readers: usize,
    pub max_writers: usize,
    pub max_nodes: usize,
    pub type_details: iox2_type_detail_t,
}

impl From<&StaticConfig> for iox2_static_config_blackboard_t {
    fn from(c: &StaticConfig) -> Self {
        Self {
            max_readers: c.max_readers(),
            max_writers: 1,
            max_nodes: c.max_nodes(),
            type_details: c.type_details().into(),
        }
    }
}
