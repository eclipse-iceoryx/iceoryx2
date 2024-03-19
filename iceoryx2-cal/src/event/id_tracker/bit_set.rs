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

use iceoryx2_bb_lock_free::mpmc::bit_set::RelocatableBitSet;
use iceoryx2_bb_log::fail;

use super::IdTracker;
use crate::event::{NotifierNotifyError, TriggerId};

impl IdTracker for RelocatableBitSet {
    fn trigger_id_max(&self) -> TriggerId {
        TriggerId::new(self.capacity() as u64)
    }

    fn add(&self, id: TriggerId) -> Result<(), NotifierNotifyError> {
        if self.trigger_id_max() <= id {
            fail!(from self, with NotifierNotifyError::TriggerIdOutOfBounds,
                "Unable to set bit {:?} since it is out of bounds (max = {:?}).",
                id, self.trigger_id_max());
        }
        self.set(id.as_u64() as usize);

        Ok(())
    }

    fn acquire_all<F: FnMut(TriggerId)>(&self, mut callback: F) {
        self.reset_all(|bit_index| callback(TriggerId::new(bit_index as u64)))
    }

    fn acquire(&self) -> Option<TriggerId> {
        match self.reset_next() {
            Some(id) => Some(TriggerId::new(id as u64)),
            None => None,
        }
    }
}
