// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_concurrency::atomic::Ordering;

use alloc::vec::Vec;

use iceoryx2_bb_concurrency::atomic::{AtomicU64, AtomicUsize};

#[derive(Debug)]
pub(crate) struct SegmentState {
    sample_reference_counter: Vec<AtomicU64>,
    payload_size: AtomicUsize,
}

impl SegmentState {
    pub(crate) fn new(number_of_samples: usize) -> Self {
        let mut sample_reference_counter = Vec::with_capacity(number_of_samples);
        for _ in 0..number_of_samples {
            sample_reference_counter.push(AtomicU64::new(0));
        }

        Self {
            sample_reference_counter,
            payload_size: AtomicUsize::new(0),
        }
    }

    pub(crate) fn set_payload_size(&self, value: usize) {
        self.payload_size.store(value, Ordering::Relaxed);
    }

    pub(crate) fn payload_size(&self) -> usize {
        self.payload_size.load(Ordering::Relaxed)
    }

    pub(crate) fn sample_index(&self, distance_to_chunk: usize) -> usize {
        debug_assert!(distance_to_chunk % self.payload_size() == 0);
        distance_to_chunk / self.payload_size()
    }

    pub(crate) fn borrow_sample(&self, distance_to_chunk: usize) -> u64 {
        self.sample_reference_counter[self.sample_index(distance_to_chunk)]
            .fetch_add(1, Ordering::Relaxed)
    }

    pub(crate) fn release_sample(&self, distance_to_chunk: usize) -> u64 {
        self.sample_reference_counter[self.sample_index(distance_to_chunk)]
            .fetch_sub(1, Ordering::Relaxed)
    }
}
