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
pub mod counting_bit_set;

use core::fmt::Debug;
use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

use crate::event::EventId;

/// Represents an activation record for a specific event.
///
/// Contains the [`EventId`] and the count of times it was activated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventActivation {
    /// The identifier of the activated event.
    pub id: EventId,
    /// The number of times the event was activated.
    pub count: u64,
}

/// Errors that can occur when attempting to activate an event with [`EventState::activate()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventStateActivateError {
    /// The provided [`EventId`] is out of bounds for the underlying event state storage.
    EventIdOutOfBounds,
}

impl core::fmt::Display for EventStateActivateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "EventStateActivateError::{self:?}")
    }
}

impl core::error::Error for EventStateActivateError {}

/// Trait defining the interface for event state management.
///
/// Event states track which events have been activated and their activation counts.
pub trait EventState: Sized + Send + Sync + Debug + ZeroCopySend + RelocatableContainer {
    /// Returns the maximum number of [`EventId`]s this state can track.
    fn max_event_count(&self) -> u64;

    /// Returns the maximum valid [`EventId`] for this state.
    fn max_event_id(&self) -> EventId;

    /// Activates an event identified by `event_id`.
    ///
    /// # Errors
    ///
    /// Returns [`EventStateActivateError::EventIdOutOfBounds`] if `event_id`
    /// exceeds the maximum event ID for this state.
    fn activate(&self, event_id: EventId) -> Result<(), EventStateActivateError>;

    /// Drains all active events, invoking `callback` for each activation record.
    ///
    /// The callback receives an [`EventActivation`] containing the [`EventId`] and
    /// its activation count. After draining, all event states are reset to inactive.
    ///
    /// # Returns
    ///
    /// The total number of event activations drained.
    fn drain<F: FnMut(EventActivation)>(&self, callback: &mut F) -> u64;
}
