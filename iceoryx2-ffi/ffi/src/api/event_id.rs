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

use iceoryx2::prelude::*;

#[derive(Clone, Copy)]
pub struct iox2_event_id_t {
    pub high: u64,
    pub low: u64,
}

impl From<iox2_event_id_t> for EventId {
    fn from(id: iox2_event_id_t) -> Self {
        EventId::new(id.low as usize)
    }
}

impl From<EventId> for iox2_event_id_t {
    fn from(id: EventId) -> Self {
        iox2_event_id_t {
            high: 0,
            low: id.as_value() as u64,
        }
    }
}
