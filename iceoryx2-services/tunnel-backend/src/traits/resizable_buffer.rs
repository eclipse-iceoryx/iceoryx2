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

/// Resizable destination for translated bytes.
///
/// The backing storage could be scratch memory, shared memory or anything
/// else. It is owned by the caller.
///
/// Resizing preserves already-written bytes but may relocate them: regions
/// returned by earlier [`resize`](ResizableBuffer::resize) calls are
/// invalidated (realloc semantics).
pub trait ResizableBuffer {
    /// Ensures at least `min_capacity` writable bytes and returns the whole
    /// writable region.
    fn resize(&mut self, min_capacity: usize) -> &mut [u8];
}

impl ResizableBuffer for alloc::vec::Vec<u8> {
    fn resize(&mut self, min_capacity: usize) -> &mut [u8] {
        if self.len() < min_capacity {
            self.resize(min_capacity, 0);
        }
        self
    }
}
