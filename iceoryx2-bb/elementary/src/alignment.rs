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

//! Represents the alignment memory can have. Ensures that the content is always
//! a power of 2 and not zero.

use crate::math::{is_power_of_2, log2_of_power_of_2};

/// Contains the alignment memory can have.
///
/// # Example
///
/// ```
/// use iceoryx2_bb_elementary::alignment::Alignment;
///
/// let my_alignment = Alignment::new(32).unwrap();
///
/// // zero is not a valid alignment
/// let broken_alignment = Alignment::new(0);
/// assert_eq!(broken_alignment, None);
///
/// // alignment must be a power of 2
/// let broken_alignment = Alignment::new(89);
/// assert_eq!(broken_alignment, None);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Alignment(u8);

impl Alignment {
    /// Creates a new [`Alignment`]. If the value is zero or not a power of 2
    /// it returns [`None`].
    pub fn new(value: usize) -> Option<Self> {
        if value == 0 || !is_power_of_2(value as _) {
            return None;
        }

        unsafe { Some(Self::new_unchecked(value)) }
    }

    /// Creates a new [`Alignment`].
    ///
    /// # Safety
    ///
    ///  * The value must not be zero
    ///  * The value must be a power of 2
    ///
    pub unsafe fn new_unchecked(value: usize) -> Self {
        Self(log2_of_power_of_2(value as _))
    }

    /// Returns the value of the [`Alignment`]
    pub fn value(&self) -> usize {
        2usize.pow(self.0 as u32)
    }
}
