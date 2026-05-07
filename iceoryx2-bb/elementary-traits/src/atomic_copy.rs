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

/// Trait for types that can be byte-wise atomically copied.
///
/// This trait provides a method to apply a callback to each offset-size pair of every field
/// of the type.
///
/// # Safety
///
/// * Implementations of this trait must ensure that the offset and size of each field are
///   calculated correctly. Otherwise, undefined behavior may occur.
pub unsafe trait AtomicCopy: Copy + 'static {
    #[doc(hidden)]
    /// Iterates over the fields of the type, calculates the offset relative to the provided
    /// base_offset, and applies the provided callback to each offset-size pair of the field.
    /// The base offset is needed because __for_each_field can be called recursively, e.g. in
    /// a nested struct. In this case, the callback must be applied to the field offsets
    /// relative to the root type and not relative to Self.
    fn __for_each_field<F: FnMut(usize, usize)>(&self, _base_offset: usize, _callback: &mut F);
}

unsafe impl AtomicCopy for usize {
    fn __for_each_field<F: FnMut(usize, usize)>(&self, base_offset: usize, callback: &mut F) {
        callback(
            base_offset.next_multiple_of(align_of::<usize>()),
            size_of::<usize>(),
        );
    }
}
unsafe impl AtomicCopy for u8 {
    fn __for_each_field<F: FnMut(usize, usize)>(&self, base_offset: usize, callback: &mut F) {
        callback(base_offset, size_of::<u8>());
    }
}
unsafe impl AtomicCopy for u16 {
    fn __for_each_field<F: FnMut(usize, usize)>(&self, base_offset: usize, callback: &mut F) {
        callback(
            base_offset.next_multiple_of(align_of::<u16>()),
            size_of::<u16>(),
        );
    }
}
unsafe impl AtomicCopy for u32 {
    fn __for_each_field<F: FnMut(usize, usize)>(&self, base_offset: usize, callback: &mut F) {
        callback(
            base_offset.next_multiple_of(align_of::<u32>()),
            size_of::<u32>(),
        );
    }
}
unsafe impl AtomicCopy for u64 {
    fn __for_each_field<F: FnMut(usize, usize)>(&self, base_offset: usize, callback: &mut F) {
        callback(
            base_offset.next_multiple_of(align_of::<u64>()),
            size_of::<u64>(),
        );
    }
}

unsafe impl AtomicCopy for isize {
    fn __for_each_field<F: FnMut(usize, usize)>(&self, base_offset: usize, callback: &mut F) {
        callback(
            base_offset.next_multiple_of(align_of::<isize>()),
            size_of::<isize>(),
        );
    }
}
unsafe impl AtomicCopy for i8 {
    fn __for_each_field<F: FnMut(usize, usize)>(&self, base_offset: usize, callback: &mut F) {
        callback(base_offset, size_of::<i8>());
    }
}
unsafe impl AtomicCopy for i16 {
    fn __for_each_field<F: FnMut(usize, usize)>(&self, base_offset: usize, callback: &mut F) {
        callback(
            base_offset.next_multiple_of(align_of::<i16>()),
            size_of::<i16>(),
        );
    }
}
unsafe impl AtomicCopy for i32 {
    fn __for_each_field<F: FnMut(usize, usize)>(&self, base_offset: usize, callback: &mut F) {
        callback(
            base_offset.next_multiple_of(align_of::<i32>()),
            size_of::<i32>(),
        );
    }
}
unsafe impl AtomicCopy for i64 {
    fn __for_each_field<F: FnMut(usize, usize)>(&self, base_offset: usize, callback: &mut F) {
        callback(
            base_offset.next_multiple_of(align_of::<i64>()),
            size_of::<i64>(),
        );
    }
}

unsafe impl AtomicCopy for f32 {
    fn __for_each_field<F: FnMut(usize, usize)>(&self, base_offset: usize, callback: &mut F) {
        callback(
            base_offset.next_multiple_of(align_of::<f32>()),
            size_of::<f32>(),
        );
    }
}
unsafe impl AtomicCopy for f64 {
    fn __for_each_field<F: FnMut(usize, usize)>(&self, base_offset: usize, callback: &mut F) {
        callback(
            base_offset.next_multiple_of(align_of::<f64>()),
            size_of::<f64>(),
        );
    }
}

unsafe impl AtomicCopy for bool {
    fn __for_each_field<F: FnMut(usize, usize)>(&self, base_offset: usize, callback: &mut F) {
        callback(
            base_offset.next_multiple_of(align_of::<bool>()),
            size_of::<bool>(),
        );
    }
}
unsafe impl AtomicCopy for char {
    fn __for_each_field<F: FnMut(usize, usize)>(&self, base_offset: usize, callback: &mut F) {
        callback(
            base_offset.next_multiple_of(align_of::<char>()),
            size_of::<char>(),
        );
    }
}
