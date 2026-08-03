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

/// The requested capacity exceeds what the buffer can provide.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ResizeError {
    /// The capacity the buffer can actually provide.
    pub available: usize,
}

impl core::fmt::Display for ResizeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ResizeError {{ available: {} }}", self.available)
    }
}

impl core::error::Error for ResizeError {}

/// Resizable destination for translated bytes.
///
/// The backing storage could be scratch memory, shared memory or anything
/// else. It is owned by the caller.
pub trait ResizableBuffer {
    /// Ensures at least `min_capacity` writable bytes and returns the whole
    /// writable region.
    ///
    /// Fails when the buffer cannot grow to `min_capacity`.
    fn resize(&mut self, min_capacity: usize) -> Result<&mut [u8], ResizeError>;
}

impl ResizableBuffer for alloc::vec::Vec<u8> {
    fn resize(&mut self, min_capacity: usize) -> Result<&mut [u8], ResizeError> {
        if self.len() < min_capacity {
            self.resize(min_capacity, 0);
        }
        Ok(self)
    }
}
