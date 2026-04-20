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

use crate::zero_copy_send::ZeroCopySend;

pub unsafe trait AtomicCopy: ZeroCopySend + Copy + 'static {
    // TODO: CallbackProgression?
    // TODO: keep it internal or make it unsafe fn for_each_field()?
    #[doc(hidden)]
    /// Iterates over the fields of the type, calculates the offset relative to the provided base_offset, and applies
    /// the provided callback to each offset-size pair of the field.
    fn __for_each_field_with_offset<F: FnMut(usize, usize)>(
        &self,
        _base_offset: usize,
        _callback: &mut F,
    ) {
    }

    // TODO
    #[doc(hidden)]
    fn __is_scalar(&self) -> bool {
        false
    }
}

unsafe impl AtomicCopy for usize {
    fn __is_scalar(&self) -> bool {
        true
    }
}
unsafe impl AtomicCopy for u8 {
    fn __is_scalar(&self) -> bool {
        true
    }
}
unsafe impl AtomicCopy for u16 {
    fn __is_scalar(&self) -> bool {
        true
    }
}
unsafe impl AtomicCopy for u32 {
    fn __is_scalar(&self) -> bool {
        true
    }
}
unsafe impl AtomicCopy for u64 {
    fn __is_scalar(&self) -> bool {
        true
    }
}

unsafe impl AtomicCopy for isize {
    fn __is_scalar(&self) -> bool {
        true
    }
}
unsafe impl AtomicCopy for i8 {
    fn __is_scalar(&self) -> bool {
        true
    }
}
unsafe impl AtomicCopy for i16 {
    fn __is_scalar(&self) -> bool {
        true
    }
}
unsafe impl AtomicCopy for i32 {
    fn __is_scalar(&self) -> bool {
        true
    }
}
unsafe impl AtomicCopy for i64 {
    fn __is_scalar(&self) -> bool {
        true
    }
}

unsafe impl AtomicCopy for f32 {
    fn __is_scalar(&self) -> bool {
        true
    }
}
unsafe impl AtomicCopy for f64 {
    fn __is_scalar(&self) -> bool {
        true
    }
}

unsafe impl AtomicCopy for bool {
    fn __is_scalar(&self) -> bool {
        true
    }
}
unsafe impl AtomicCopy for char {
    fn __is_scalar(&self) -> bool {
        true
    }
}

unsafe impl<T: AtomicCopy, const N: usize> AtomicCopy for [T; N] {}

unsafe impl<T1: AtomicCopy, T2: AtomicCopy> AtomicCopy for (T1, T2) {}
unsafe impl<T1: AtomicCopy, T2: AtomicCopy, T3: AtomicCopy> AtomicCopy for (T1, T2, T3) {}
unsafe impl<T1: AtomicCopy, T2: AtomicCopy, T3: AtomicCopy, T4: AtomicCopy> AtomicCopy
    for (T1, T2, T3, T4)
{
}
unsafe impl<T1: AtomicCopy, T2: AtomicCopy, T3: AtomicCopy, T4: AtomicCopy, T5: AtomicCopy>
    AtomicCopy for (T1, T2, T3, T4, T5)
{
}
unsafe impl<
        T1: AtomicCopy,
        T2: AtomicCopy,
        T3: AtomicCopy,
        T4: AtomicCopy,
        T5: AtomicCopy,
        T6: AtomicCopy,
    > AtomicCopy for (T1, T2, T3, T4, T5, T6)
{
}
unsafe impl<
        T1: AtomicCopy,
        T2: AtomicCopy,
        T3: AtomicCopy,
        T4: AtomicCopy,
        T5: AtomicCopy,
        T6: AtomicCopy,
        T7: AtomicCopy,
    > AtomicCopy for (T1, T2, T3, T4, T5, T6, T7)
{
}
unsafe impl<
        T1: AtomicCopy,
        T2: AtomicCopy,
        T3: AtomicCopy,
        T4: AtomicCopy,
        T5: AtomicCopy,
        T6: AtomicCopy,
        T7: AtomicCopy,
        T8: AtomicCopy,
    > AtomicCopy for (T1, T2, T3, T4, T5, T6, T7, T8)
{
}
