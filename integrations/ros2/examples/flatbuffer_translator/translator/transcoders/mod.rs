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

mod joint_readings;

use core::ops::{Deref, DerefMut};

use flatbuffers::Allocator;
use iceoryx2_link_backend::wire::region::{BackwardRegion, ResizeError};

pub use joint_readings::JointReadingsTranscoder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranscodeFailure {
    /// The payload is not a flatbuffer of the expected type.
    InvalidFlatbuffer,
    Serialize,
    Deserialize,
}

impl core::fmt::Display for TranscodeFailure {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TranscodeFailure::{self:?}")
    }
}

impl core::error::Error for TranscodeFailure {}

/// A backward region as the memory a flatbuffer is built in.
pub struct RegionAllocator<B: BackwardRegion>(pub B);

impl<B: BackwardRegion> Deref for RegionAllocator<B> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl<B: BackwardRegion> DerefMut for RegionAllocator<B> {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

// SAFETY: a backward region keeps its bytes at the end when it is resized,
// which is where the builder expects them after a grow.
unsafe impl<B: BackwardRegion> Allocator for RegionAllocator<B> {
    type Error = ResizeError;

    fn grow_downwards(&mut self) -> Result<(), ResizeError> {
        let grown = core::cmp::max(1, 2 * self.0.len());
        self.0.resize(grown)
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}
