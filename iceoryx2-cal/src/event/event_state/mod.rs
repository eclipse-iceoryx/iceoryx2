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

pub mod bit_set;

use core::fmt::Debug;
use iceoryx2_bb_container::queue::RelocatableContainer;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

use crate::event::EventId;

pub struct EventActivation {
    pub event_id: EventId,
    pub count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventStateActivateError {
    EventIdOutOfBounds,
}

impl core::fmt::Display for EventStateActivateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "EventStateActivateError::{self:?}")
    }
}

impl core::error::Error for EventStateActivateError {}

pub trait EventState: Sized + Send + Sync + Debug + ZeroCopySend + RelocatableContainer {
    fn max_event_id(&self) -> EventId;
    fn activate(&self, event_id: EventId) -> Result<(), EventStateActivateError>;
    fn drain<F: FnMut(EventActivation)>(&self, callback: F) -> u64;
}
