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

use alloc::vec::Vec;

/// A region cannot be provided for the requested length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnsupportedLength;

impl core::fmt::Display for UnsupportedLength {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "UnsupportedLength")
    }
}

impl core::error::Error for UnsupportedLength {}

/// A writable byte region whose length is set by its writer.
pub trait Region {
    /// Resize the region for `len` bytes.
    fn for_length(&mut self, len: usize) -> Result<&mut [u8], UnsupportedLength>;
}

impl<R: Region + ?Sized> Region for &mut R {
    fn for_length(&mut self, len: usize) -> Result<&mut [u8], UnsupportedLength> {
        (**self).for_length(len)
    }
}

impl Region for Vec<u8> {
    fn for_length(&mut self, len: usize) -> Result<&mut [u8], UnsupportedLength> {
        self.resize(len, 0);
        Ok(self)
    }
}

impl Region for [u8] {
    fn for_length(&mut self, len: usize) -> Result<&mut [u8], UnsupportedLength> {
        match len == self.len() {
            true => Ok(self),
            false => Err(UnsupportedLength),
        }
    }
}
