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

use iceoryx2::{prelude::EventId, service::static_config::event::StaticConfig};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct iox2_static_config_event_t {
    pub max_notifiers: usize,
    pub max_listeners: usize,
    pub max_nodes: usize,
    pub event_id_max_value: usize,
    pub notifier_dead_event: usize,
    pub has_notifier_dead_event: bool,
    pub notifier_dropped_event: usize,
    pub has_notifier_dropped_event: bool,
    pub notifier_created_event: usize,
    pub has_notifier_created_event: bool,
    pub deadline_seconds: u64,
    pub deadline_nanoseconds: u32,
    pub has_deadline: bool,
}

impl From<&StaticConfig> for iox2_static_config_event_t {
    fn from(c: &StaticConfig) -> Self {
        Self {
            max_notifiers: c.max_notifiers(),
            max_listeners: c.max_listeners(),
            max_nodes: c.max_nodes(),
            event_id_max_value: c.event_id_max_value(),
            notifier_dead_event: c
                .notifier_dead_event()
                .unwrap_or(EventId::new(0))
                .as_value(),
            has_notifier_dead_event: c.notifier_dead_event().is_some(),
            notifier_dropped_event: c
                .notifier_dropped_event()
                .unwrap_or(EventId::new(0))
                .as_value(),
            has_notifier_dropped_event: c.notifier_dropped_event().is_some(),
            notifier_created_event: c
                .notifier_created_event()
                .unwrap_or(EventId::new(0))
                .as_value(),
            has_notifier_created_event: c.notifier_created_event().is_some(),
            deadline_seconds: c.deadline().map(|v| v.as_secs()).unwrap_or(0),
            deadline_nanoseconds: c.deadline().map(|v| v.subsec_nanos()).unwrap_or(0),
            has_deadline: c.deadline().is_some(),
        }
    }
}
