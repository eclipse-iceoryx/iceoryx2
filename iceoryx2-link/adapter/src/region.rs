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

/// Why a region could not take a length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeError {
    /// The region has one length and this is not it.
    NotResizable,
    /// No sample of this length can exist for the service.
    Malformed,
    /// The region could not be backed, no sample could be loaned.
    Exhausted,
}

impl core::fmt::Display for ResizeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ResizeError::{self:?}")
    }
}

impl core::error::Error for ResizeError {}

/// A writable byte region whose length is set by its writer.
pub trait Region {
    /// The region at exactly `len` bytes.
    fn for_length(&mut self, len: usize) -> Result<&mut [u8], ResizeError>;
}

impl<R: Region + ?Sized> Region for &mut R {
    fn for_length(&mut self, len: usize) -> Result<&mut [u8], ResizeError> {
        (**self).for_length(len)
    }
}

impl Region for Vec<u8> {
    fn for_length(&mut self, len: usize) -> Result<&mut [u8], ResizeError> {
        self.resize(len, 0);
        Ok(self)
    }
}

/// A region of one length.
impl Region for [u8] {
    fn for_length(&mut self, len: usize) -> Result<&mut [u8], ResizeError> {
        match len == self.len() {
            true => Ok(self),
            false => Err(ResizeError::NotResizable),
        }
    }
}
