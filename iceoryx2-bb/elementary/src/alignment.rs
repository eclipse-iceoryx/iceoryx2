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
pub struct Alignment(usize);

impl Alignment {
    pub const ALIGN_1: Alignment = Alignment(1);
    pub const ALIGN_2: Alignment = Alignment(2);
    pub const ALIGN_4: Alignment = Alignment(4);
    pub const ALIGN_8: Alignment = Alignment(8);
    pub const ALIGN_16: Alignment = Alignment(16);
    pub const ALIGN_32: Alignment = Alignment(32);
    pub const ALIGN_64: Alignment = Alignment(64);
    pub const ALIGN_128: Alignment = Alignment(128);
    pub const ALIGN_256: Alignment = Alignment(256);
    pub const ALIGN_512: Alignment = Alignment(512);
    pub const ALIGN_1024: Alignment = Alignment(1024);
    pub const ALIGN_2048: Alignment = Alignment(2048);
    pub const ALIGN_4096: Alignment = Alignment(4096);

    /// Creates a new [`Alignment`]. If the value is zero or not a power of 2
    /// it returns [`None`].
    pub fn new(value: usize) -> Option<Self> {
        (value.is_power_of_two()).then(|| unsafe { Self::new_unchecked(value) })
    }

    /// Creates a new [`Alignment`].
    ///
    /// # Safety
    ///
    ///  * The value must not be zero
    ///  * The value must be a power of 2
    ///
    pub const unsafe fn new_unchecked(value: usize) -> Self {
        Self(value)
    }

    /// Returns the value of the [`Alignment`]
    pub const fn value(&self) -> usize {
        self.0
    }
}
