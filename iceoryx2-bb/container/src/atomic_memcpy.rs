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
use iceoryx2_log::fail;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum AtomicMemcpyError {
    AtomicMemcpyCreateError,
}

// TODO: better name
// TODO: get rid of size parameter
#[repr(C)]
pub struct AtomicMemcpy<T: Copy, const SIZE: usize> {
    // data: [AtomicU8; size_of::<T>()],
    // data: [AtomicU8; Self::LEN],
    data: [AtomicU8; SIZE],
    _inner_type: PhantomData<T>,
}

// TODO: impl Send, ZeroCopySend?

impl<T: Copy, const SIZE: usize> AtomicMemcpy<T, SIZE> {
    // const LEN: usize = size_of::<T>();

    pub fn new(value: T) -> Result<Self, AtomicMemcpyError> {
        if size_of::<T>() != SIZE {
            fail!(from "AtomicMemcpy::new()", with AtomicMemcpyError::AtomicMemcpyCreateError,
                "size_of::<T>() and SIZE must be equal.");
        }
        let value_ptr = (&value as *const T) as *const u8;
        Ok(Self {
            data: core::array::from_fn(|i| AtomicU8::new(unsafe { *value_ptr.add(i) })),
            _inner_type: PhantomData,
        })
    }

    pub unsafe fn read(&self) -> MaybeUninit<T> {
        let mut data: MaybeUninit<T> = MaybeUninit::uninit();
        let data_ptr = data.as_mut_ptr() as *mut u8;
        for (i, val) in self.data.iter().enumerate() {
            *data_ptr.add(i) = val.load(Ordering::Relaxed);
        }
        data
    }

    pub unsafe fn write(&mut self, value: T) {
        let value_ptr = (&value as *const T) as *const u8;
        for i in 0..SIZE {
            self.data[i].store(*value_ptr.add(i), Ordering::Relaxed);
        }
    }
}
