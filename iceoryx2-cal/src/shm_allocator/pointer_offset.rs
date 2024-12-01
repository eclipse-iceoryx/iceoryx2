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

use std::fmt::Debug;

pub type SegmentIdUnderlyingType = u8;

/// Defines the [`SegmentId`] of a [`SharedMemory`](crate::shared_memory::SharedMemory)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentId(SegmentIdUnderlyingType);

impl SegmentId {
    /// Creates a new [`SegmentId`] from a given value.
    pub const fn new(value: SegmentIdUnderlyingType) -> Self {
        Self(value)
    }

    /// Returns the underlying value of the [`SegmentId`]
    pub const fn value(&self) -> SegmentIdUnderlyingType {
        self.0
    }

    /// Returns the maximum value the [`SegmentId`] supports.
    pub const fn max_segment_id() -> SegmentIdUnderlyingType {
        SegmentIdUnderlyingType::MAX
    }
}

/// An offset to a [`SharedMemory`](crate::shared_memory::SharedMemory) address. It requires the
///
/// [`SharedMemory::payload_start_address()`](crate::shared_memory::SharedMemory::payload_start_address())
/// of the corresponding [`SharedMemory`](crate::shared_memory::SharedMemory) to be converted into
/// an actual pointer.
///
/// Contains the offset and the corresponding [`SegmentId`].
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PointerOffset(u64);

impl PointerOffset {
    /// Creates a new [`PointerOffset`] from the given offset value with the [`SegmentId`] == 0.
    pub const fn new(offset: usize) -> PointerOffset {
        const SEGMENT_ID: u64 = 0;
        Self((offset as u64) << (SegmentIdUnderlyingType::BITS) | SEGMENT_ID)
    }

    /// Sets the [`SegmentId`] of the [`PointerOffset`].
    pub fn set_segment_id(&mut self, value: SegmentId) {
        self.0 &= !((1u64 << SegmentIdUnderlyingType::BITS) - 1);
        self.0 |= value.0 as u64;
    }

    /// Returns the offset.
    pub const fn offset(&self) -> usize {
        (self.0 >> (SegmentIdUnderlyingType::BITS)) as usize
    }

    /// Returns the [`SegmentId`].
    pub const fn segment_id(&self) -> SegmentId {
        SegmentId((self.0 & ((1u64 << SegmentIdUnderlyingType::BITS) - 1)) as u8)
    }
}

impl Debug for PointerOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PointerOffset {{ offset: {}, segment_id: {:?} }}",
            self.offset(),
            self.segment_id()
        )
    }
}
