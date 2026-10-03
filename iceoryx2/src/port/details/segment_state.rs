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
use iceoryx2_bb_concurrency::cell::OnceCell;

#[derive(Debug)]
pub(crate) struct SegmentState {
    number_of_chunks: usize,
    chunk_reference_counter: OnceCell<Vec<AtomicU64>>,
    payload_size: AtomicUsize,
}

impl SegmentState {
    /// Creates the state of a segment that exists from the start, with its counters allocated.
    pub(crate) fn new(number_of_chunks: usize) -> Self {
        let state = Self::new_unallocated(number_of_chunks);
        state.chunk_reference_counter();
        state
    }

    /// Creates the state of a segment that is only created when the data segment grows. Its
    /// counters are allocated by the loan that grows the data segment.
    pub(crate) fn new_unallocated(number_of_chunks: usize) -> Self {
        Self {
            number_of_chunks,
            chunk_reference_counter: OnceCell::new(),
            payload_size: AtomicUsize::new(0),
        }
    }

    fn chunk_reference_counter(&self) -> &[AtomicU64] {
        self.chunk_reference_counter.get_or_init(|| {
            let mut counters = Vec::with_capacity(self.number_of_chunks);
            for _ in 0..self.number_of_chunks {
                counters.push(AtomicU64::new(0));
            }
            counters
        })
    }

    pub(crate) fn set_payload_size(&self, value: usize) {
        self.payload_size.store(value, Ordering::Relaxed);
    }

    pub(crate) fn payload_size(&self) -> usize {
        self.payload_size.load(Ordering::Relaxed)
    }

    pub(crate) fn chunk_index(&self, distance_to_chunk: usize) -> usize {
        debug_assert!(distance_to_chunk.is_multiple_of(self.payload_size()));
        distance_to_chunk / self.payload_size()
    }

    pub(crate) fn borrow_chunk(&self, distance_to_chunk: usize) -> u64 {
        self.chunk_reference_counter()[self.chunk_index(distance_to_chunk)]
            .fetch_add(1, Ordering::Relaxed)
    }

    pub(crate) fn release_chunk(&self, distance_to_chunk: usize) -> u64 {
        self.chunk_reference_counter()[self.chunk_index(distance_to_chunk)]
            .fetch_sub(1, Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_reference_counters_of_a_new_segment_are_allocated_at_creation() {
        const NUMBER_OF_CHUNKS: usize = 8;
        let sut = SegmentState::new(NUMBER_OF_CHUNKS);

        assert_eq!(
            sut.chunk_reference_counter.get().map(|v| v.len()),
            Some(NUMBER_OF_CHUNKS)
        );
    }

    #[test]
    fn chunk_reference_counters_of_an_unallocated_segment_are_allocated_on_first_borrow() {
        const NUMBER_OF_CHUNKS: usize = 8;
        const PAYLOAD_SIZE: usize = 16;
        let sut = SegmentState::new_unallocated(NUMBER_OF_CHUNKS);
        sut.set_payload_size(PAYLOAD_SIZE);

        assert!(sut.chunk_reference_counter.get().is_none());

        assert_eq!(sut.borrow_chunk(3 * PAYLOAD_SIZE), 0);
        assert_eq!(
            sut.chunk_reference_counter.get().map(|v| v.len()),
            Some(NUMBER_OF_CHUNKS)
        );
        assert_eq!(sut.release_chunk(3 * PAYLOAD_SIZE), 1);
    }
}
