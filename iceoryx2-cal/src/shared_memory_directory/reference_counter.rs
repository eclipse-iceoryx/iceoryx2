// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

use core::sync::atomic::Ordering;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU64;

struct ReferenceCounterDetails {
    reference_counter: u64,
    is_initialized: bool,
    is_persistent: bool,
    to_be_removed: bool,
}

impl ReferenceCounterDetails {
    fn new(value: u64) -> Self {
        Self {
            reference_counter: (value << 3) >> 3,
            to_be_removed: ((value << 2) >> 63) == 1,
            is_initialized: ((value << 1) >> 63) == 1,
            is_persistent: (value >> 63) == 1,
        }
    }

    fn compact_value(&self) -> u64 {
        let to_be_removed = if self.to_be_removed { 1u64 } else { 0u64 };
        let is_persistent = if self.is_persistent { 1u64 } else { 0u64 };
        let is_initialized = if self.is_initialized { 1u64 } else { 0u64 };
        let reference_counter = (self.reference_counter << 3) >> 3;

        (is_persistent << 63) | (is_initialized << 62) | (to_be_removed << 61) | reference_counter
    }
}

#[derive(Debug)]
pub(crate) struct ReferenceCounter(IoxAtomicU64);

impl ReferenceCounter {
    pub(crate) const fn new(value: u64) -> ReferenceCounter {
        ReferenceCounter(IoxAtomicU64::new(value))
    }

    pub(crate) fn reset(&self) {
        self.0.swap(0, Ordering::Relaxed);
    }

    pub(crate) fn increment_ref_counter(&self) -> u64 {
        let mut current_value = self.0.load(Ordering::Relaxed);

        loop {
            let mut details = ReferenceCounterDetails::new(current_value);
            details.reference_counter += 1;

            match self.0.compare_exchange(
                current_value,
                details.compact_value(),
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(v) => return v,
                Err(v) => current_value = v,
            }
        }
    }

    pub(crate) fn increment_ref_counter_when_initialized(&self) -> bool {
        let mut current_value = self.0.load(Ordering::Relaxed);

        loop {
            let mut details = ReferenceCounterDetails::new(current_value);

            if !(!details.to_be_removed
                && details.is_initialized
                && (details.reference_counter > 0 || details.is_persistent))
            {
                return false;
            }

            details.reference_counter += 1;

            match self.0.compare_exchange(
                current_value,
                details.compact_value(),
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(_) => return true,
                Err(v) => current_value = v,
            }
        }
    }

    pub(crate) fn increment_ref_counter_when_exist(&self) -> bool {
        let mut current_value = self.0.load(Ordering::Relaxed);

        loop {
            let mut details = ReferenceCounterDetails::new(current_value);

            if !(!details.to_be_removed && (details.reference_counter > 0 || details.is_persistent))
            {
                return false;
            }

            details.reference_counter += 1;

            match self.0.compare_exchange(
                current_value,
                details.compact_value(),
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(_) => return true,
                Err(v) => current_value = v,
            }
        }
    }

    pub(crate) fn decrement_ref_counter(&self) -> bool {
        let mut current_value = self.0.load(Ordering::Relaxed);

        loop {
            let mut details = ReferenceCounterDetails::new(current_value);
            details.reference_counter -= 1;

            let can_be_removed = details.reference_counter == 0 && !details.is_persistent;

            match self.0.compare_exchange(
                current_value,
                details.compact_value(),
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return can_be_removed,
                Err(v) => current_value = v,
            }
        }
    }

    pub(crate) fn set_initialized_bit(&self, is_set: bool) {
        let mut current_value = self.0.load(Ordering::Relaxed);

        loop {
            let mut details = ReferenceCounterDetails::new(current_value);
            if details.is_initialized == is_set {
                return;
            }

            details.is_initialized = is_set;
            match self.0.compare_exchange(
                current_value,
                details.compact_value(),
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return,
                Err(v) => current_value = v,
            }
        }
    }

    pub(crate) fn is_initialized(&self) -> bool {
        ReferenceCounterDetails::new(self.0.load(Ordering::Relaxed)).is_initialized
    }

    pub(crate) fn is_persistent(&self) -> bool {
        ReferenceCounterDetails::new(self.0.load(Ordering::Relaxed)).is_persistent
    }

    pub(crate) fn to_be_removed(&self) {
        let mut current_value = self.0.load(Ordering::Relaxed);

        loop {
            let mut details = ReferenceCounterDetails::new(current_value);
            if details.to_be_removed {
                return;
            }

            details.to_be_removed = true;
            match self.0.compare_exchange(
                current_value,
                details.compact_value(),
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return,
                Err(v) => current_value = v,
            }
        }
    }

    pub(crate) fn set_persistency_bit(&self, is_set: bool) {
        let mut current_value = self.0.load(Ordering::Relaxed);

        loop {
            let mut details = ReferenceCounterDetails::new(current_value);
            if details.is_persistent == is_set {
                return;
            }

            details.is_persistent = is_set;
            match self.0.compare_exchange(
                current_value,
                details.compact_value(),
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return,
                Err(v) => current_value = v,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use core::sync::atomic::Ordering;

    use iceoryx2_bb_testing::assert_that;

    use crate::shared_memory_directory::reference_counter::{
        ReferenceCounter, ReferenceCounterDetails,
    };

    #[test]
    fn reference_counter_details_work() {
        let sut = ReferenceCounterDetails {
            reference_counter: 1234,
            is_initialized: true,
            is_persistent: false,
            to_be_removed: false,
        };

        let new_sut = ReferenceCounterDetails::new(sut.compact_value());

        assert_that!(new_sut.reference_counter, eq 1234);
        assert_that!(new_sut.is_initialized, eq true);
        assert_that!(new_sut.is_persistent, eq false);
        assert_that!(new_sut.to_be_removed, eq false);

        let sut = ReferenceCounterDetails {
            reference_counter: 987654321,
            is_initialized: false,
            is_persistent: true,
            to_be_removed: true,
        };

        let new_sut = ReferenceCounterDetails::new(sut.compact_value());

        assert_that!(new_sut.reference_counter, eq 987654321);
        assert_that!(new_sut.is_initialized, eq false);
        assert_that!(new_sut.is_persistent, eq true);
        assert_that!(new_sut.to_be_removed, eq true);
    }

    #[test]
    fn reference_counter_works() {
        let sut = ReferenceCounter::new(0);

        for _ in 0..2345 {
            sut.increment_ref_counter();
        }

        sut.set_initialized_bit(false);
        sut.set_persistency_bit(true);

        let details = ReferenceCounterDetails::new(sut.0.load(Ordering::Relaxed));

        assert_that!(details.reference_counter, eq 2345);
        assert_that!(details.is_initialized, eq false);
        assert_that!(details.is_persistent, eq true);
    }
}
