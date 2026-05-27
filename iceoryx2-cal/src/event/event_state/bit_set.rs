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

use iceoryx2_bb_lock_free::mpmc::bit_set::RelocatableBitSet;
use iceoryx2_log::fail;

use crate::event::event_state::{EventActivation, EventId, EventState, EventStateActivateError};

impl EventState for RelocatableBitSet {
    fn max_event_id(&self) -> EventId {
        EventId::new(self.capacity() as u64 - 1)
    }

    fn activate(&self, event_id: EventId) -> Result<(), EventStateActivateError> {
        if self.max_event_id() < event_id {
            fail!(from self, with EventStateActivateError::IdOutOfBounds,
                "Unable to activate {event_id:?} since it is out of bounds (max = {:?}).", self.max_event_id())
        }
        self.set(event_id.as_value() as usize);

        Ok(())
    }

    fn drain<F: FnMut(EventActivation)>(&self, mut callback: F) -> u64 {
        let mut counter = 0;
        self.reset_all(|bit_index| {
            counter += 1;
            callback(EventActivation {
                event_id: EventId::new(bit_index as u64),
                count: 1,
            });
        });
        counter
    }
}
