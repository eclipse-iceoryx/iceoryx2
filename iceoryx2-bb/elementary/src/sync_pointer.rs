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

use core::fmt::Debug;

use iceoryx2_bb_elementary_traits::pointer::Pointer;

#[derive(Debug)]
/// Helper struct to keep [`Send`] and [`Sync`] auto implementation in structs that
/// use pointers and are otherwise threadsafe.
pub struct SyncPointer<T: Debug>(*mut T);

unsafe impl<T: Debug> Send for SyncPointer<T> {}
unsafe impl<T: Debug> Sync for SyncPointer<T> {}

impl<T: Debug> SyncPointer<T> {
    pub fn new(ptr: *mut T) -> Self {
        Self(ptr)
    }
}

impl<T: Debug> Pointer<T> for SyncPointer<T> {
    fn as_ptr(&self) -> *const T {
        self.0
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        self.0
    }

    fn is_initialized(&self) -> bool {
        true
    }
}
