// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use core::ffi::c_char;

use iceoryx2::service::static_config::StaticConfig;

use crate::{iox2_messaging_pattern_e, IOX2_SERVICE_ID_LENGTH, IOX2_SERVICE_NAME_LENGTH};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct iox2_static_config_t {
    pub id: [c_char; IOX2_SERVICE_ID_LENGTH],
    pub name: [c_char; IOX2_SERVICE_NAME_LENGTH],
    pub messaging_pattern: iox2_messaging_pattern_e,
}

impl From<StaticConfig> for iox2_static_config_t {
    fn from(value: StaticConfig) -> Self {
        Self {
            id: core::array::from_fn(|n| {
                if n < value.uuid().as_bytes().len() {
                    value.uuid().as_bytes()[n] as _
                } else {
                    0
                }
            }),
            name: core::array::from_fn(|n| {
                if n < value.name().as_bytes().len() {
                    value.name().as_bytes()[n] as _
                } else {
                    0
                }
            }),
            messaging_pattern: value.messaging_pattern().into(),
        }
    }
}
