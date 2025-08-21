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

use iceoryx2::service::static_config::publish_subscribe::StaticConfig;

use crate::iox2_message_type_details_t;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct iox2_static_config_publish_subscribe_t {
    pub max_subscribers: usize,
    pub max_publishers: usize,
    pub max_nodes: usize,
    pub history_size: usize,
    pub subscriber_max_buffer_size: usize,
    pub subscriber_max_borrowed_samples: usize,
    pub enable_safe_overflow: bool,
    pub message_type_details: iox2_message_type_details_t,
}

impl From<&StaticConfig> for iox2_static_config_publish_subscribe_t {
    fn from(c: &StaticConfig) -> Self {
        Self {
            max_subscribers: c.max_subscribers(),
            max_publishers: c.max_publishers(),
            max_nodes: c.max_nodes(),
            history_size: c.history_size(),
            subscriber_max_buffer_size: c.subscriber_max_buffer_size(),
            subscriber_max_borrowed_samples: c.subscriber_max_borrowed_samples(),
            enable_safe_overflow: c.has_safe_overflow(),
            message_type_details: c.message_type_details().into(),
        }
    }
}
