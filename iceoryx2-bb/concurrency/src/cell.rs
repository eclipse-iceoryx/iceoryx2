// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_elementary_traits::placement_default::PlacementDefault;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_pal_concurrency_sync::cell::Cell as InternalCell;
use iceoryx2_pal_concurrency_sync::cell::OnceCell as InternalOnceCell;
use iceoryx2_pal_concurrency_sync::cell::Ref;
use iceoryx2_pal_concurrency_sync::cell::RefCell as InternalRefCell;
use iceoryx2_pal_concurrency_sync::cell::RefMut;
use iceoryx2_pal_concurrency_sync::cell::UnsafeCell as InternalUnsafeCell;

#[derive(Default)]
#[repr(transparent)]
pub struct Cell<T: ?Sized>(InternalCell<T>);

impl<T: Default> PlacementDefault for Cell<T> {
    unsafe fn placement_default(ptr: *mut Self) {
        ptr.write(Cell::default());
    }
}

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct OnceCell<T>(InternalOnceCell<T>);

impl<T> Cell<T> {
    pub const fn new(value: T) -> Self {
        Self(InternalCell::new(value))
    }

    pub fn as_ptr(&self) -> *mut T {
        self.0.as_ptr()
    }

    pub fn set(&self, value: T) {
        self.0.set(value)
    }
}

impl<T: Default> PlacementDefault for OnceCell<T> {
    unsafe fn placement_default(ptr: *mut Self) {
        ptr.write(OnceCell::default());
    }
}

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct RefCell<T>(InternalRefCell<T>);

impl<T> RefCell<T> {
    pub const fn new(value: T) -> Self {
        Self(InternalRefCell::new(value))
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.0.borrow_mut()
    }
}

impl<T: Default> PlacementDefault for RefCell<T> {
    unsafe fn placement_default(ptr: *mut Self) {
        ptr.write(RefCell::default());
    }
}

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct UnsafeCell<T>(InternalUnsafeCell<T>);

impl<T> UnsafeCell<T> {
    pub const fn new(value: T) -> Self {
        Self(InternalUnsafeCell::new(value))
    }

    pub fn get(&self) -> *mut T {
        self.0.get()
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.0.get_mut()
    }
}

impl<T: Default> PlacementDefault for UnsafeCell<T> {
    unsafe fn placement_default(ptr: *mut Self) {
        ptr.write(UnsafeCell::default());
    }
}

unsafe impl<T: ZeroCopySend> ZeroCopySend for UnsafeCell<T> {}
