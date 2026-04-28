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

use core::marker::PhantomData;
use core::mem::MaybeUninit;
use iceoryx2_bb_concurrency::atomic::{AtomicU8, Ordering};
use iceoryx2_bb_elementary_traits::{atomic_copy::AtomicCopy, zero_copy_send::ZeroCopySend};
use iceoryx2_log::fail;

/// Failures caused by [`ByteAtomic::new()`]
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum ByteAtomicError {
    /// The size of the passed value and SIZE do not match.
    SizesDoNotMatch,
}

/// A wrapper type that provides byte-wise atomic read and write accesses on the inner type `T`.
/// It only guarantees atomicity at the byte level; it does not provide higher-level thread-
/// safety guarantees. Users must still enforce proper synchronization, i.e. torn-writes and
/// torn-reads are still possible and must be handled by the user. The wrapper does only ensure
/// that the memory copy does not cause undefined behavior, but does not care about data integrity.
#[repr(C)]
pub struct ByteAtomic<T: AtomicCopy, const SIZE: usize> {
    data: [AtomicU8; SIZE],
    _inner_type: PhantomData<T>,
}

unsafe impl<T: AtomicCopy + ZeroCopySend, const SIZE: usize> ZeroCopySend for ByteAtomic<T, SIZE> {}

impl<T: AtomicCopy, const SIZE: usize> ByteAtomic<T, SIZE> {
    /// Creates a new [`ByteAtomic`] that contains the passed value. It fails when the size
    /// of the value and `SIZE` do not match.
    pub fn new(value: T) -> Result<Self, ByteAtomicError> {
        // TODO: remove the following check once size_of::<T>() can be directly used in the
        // struct definition
        if size_of::<T>() != SIZE {
            fail!(from "ByteAtomic::new()", with ByteAtomicError::SizesDoNotMatch,
                "size_of::<T>() and SIZE must be equal.");
        }

        let value_ptr = (&value as *const T) as *const u8;

        // The passed value may contain padding bytes. Reading or copying these padding bytes
        // would lead to undefined behavior. Therefore, we first set all bytes to zero and
        // then copy only the fields, i.e. the initialized bytes, of the passed value.
        let mut bytes = [0u8; SIZE];
        value.__for_each_field(0, &mut |offset, size| {
            for (i, byte) in bytes.iter_mut().enumerate().skip(offset).take(size) {
                *byte = unsafe { *value_ptr.add(i) };
            }
        });
        Ok(Self {
            data: bytes.map(AtomicU8::new),
            _inner_type: PhantomData,
        })
    }

    /// Copies the stored value byte-wise into a [`MaybeUninit<T>`].
    ///
    /// # Safety
    ///
    /// * When the value is concurrently written to, torn-reads are possible. The user must take care
    ///   of the data integrity.
    pub unsafe fn read(&self) -> MaybeUninit<T> {
        let mut data: MaybeUninit<T> = MaybeUninit::uninit();
        let data_ptr = data.as_mut_ptr() as *mut u8;
        for (i, item) in self.data.iter().enumerate() {
            unsafe {
                *data_ptr.add(i) = item.load(Ordering::Relaxed);
            }
        }
        data
    }

    /// Stores the passed value byte-wise atomically.
    ///
    /// # Safety
    ///
    /// * When used concurrently, torn-writes and torn-reads are possible. The user must take care
    ///   of the data integrity.
    pub unsafe fn write(&self, value: T) {
        let value_ptr = (&value as *const T) as *const u8;
        value.__for_each_field(0, &mut |offset, size| {
            for i in offset..offset + size {
                self.data[i].store(unsafe { *value_ptr.add(i) }, Ordering::Relaxed);
            }
        });
    }
}
