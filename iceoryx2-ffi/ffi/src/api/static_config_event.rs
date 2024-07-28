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

use iceoryx2::service::static_config::event::StaticConfig;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct iox2_static_config_event_t {
    pub max_notifiers: usize,
    pub max_listeners: usize,
    pub max_nodes: usize,
    pub event_id_max_value: usize,
}

impl From<&StaticConfig> for iox2_static_config_event_t {
    fn from(c: &StaticConfig) -> Self {
        Self {
            max_notifiers: c.max_notifiers(),
            max_listeners: c.max_listeners(),
            max_nodes: c.max_nodes(),
            event_id_max_value: c.event_id_max_value(),
        }
    }
}
