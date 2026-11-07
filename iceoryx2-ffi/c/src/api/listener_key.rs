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

#![allow(non_camel_case_types)]

use crate::api::{AssertNonNullHandle, HandleToType};
use iceoryx2::port::notifier::ListenerKey;
use iceoryx2_ffi_macros::iceoryx2_ffi;

/// Storage for an owned listener key.
#[repr(C)]
#[repr(align(8))]
pub struct iox2_listener_key_storage_t {
    // Capacity for Option<ListenerKey>; its layout varies by target.
    // The FFI macro checks that storage size and alignment are sufficient.
    internal: [u8; 32],
}

/// An owned, retainable key identifying a listener connection. Clone it before
/// sharing ownership and drop every owned copy with `iox2_listener_key_drop`.
#[repr(C)]
#[iceoryx2_ffi(ListenerKey)]
pub struct iox2_listener_key_t {
    pub value: iox2_listener_key_storage_t,
    pub(super) deleter: fn(*mut iox2_listener_key_t),
}

impl iox2_listener_key_t {
    fn init(&mut self, value: ListenerKey, deleter: fn(*mut iox2_listener_key_t)) {
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_listener_key_h_t;
/// Owning listener key handle.
pub type iox2_listener_key_h = *mut iox2_listener_key_h_t;
/// Non-owning pointer to a listener key handle.
pub type iox2_listener_key_h_ref = *const iox2_listener_key_h;

impl AssertNonNullHandle for iox2_listener_key_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_listener_key_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_listener_key_h {
    type Target = *mut iox2_listener_key_t;
    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_listener_key_h_ref {
    type Target = *mut iox2_listener_key_t;
    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

pub(super) unsafe fn init_listener_key(
    value: ListenerKey,
    key_struct_ptr: *mut iox2_listener_key_t,
    key_handle_ptr: *mut iox2_listener_key_h,
) {
    debug_assert!(!key_handle_ptr.is_null());
    fn no_op(_: *mut iox2_listener_key_t) {}
    let mut deleter: fn(*mut iox2_listener_key_t) = no_op;
    let mut storage_ptr = key_struct_ptr;
    if storage_ptr.is_null() {
        storage_ptr = iox2_listener_key_t::alloc();
        deleter = iox2_listener_key_t::dealloc;
    }
    debug_assert!(!storage_ptr.is_null());
    unsafe {
        (*storage_ptr).init(value, deleter);
        *key_handle_ptr = (*storage_ptr).as_handle();
    }
}

// BEGIN C API

/// Creates another independently owned copy of a listener key. If
/// `key_struct_ptr` is null, storage is allocated on the heap; otherwise it
/// must point to uninitialized storage kept alive until the copy is dropped.
///
/// # Safety
/// `key` must point to a live listener key handle and `key_handle_ptr` must
/// point to writable output storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_listener_key_clone(
    key: iox2_listener_key_h_ref,
    key_struct_ptr: *mut iox2_listener_key_t,
    key_handle_ptr: *mut iox2_listener_key_h,
) {
    key.assert_non_null();
    unsafe {
        init_listener_key(
            *(*key.as_type()).value.as_ref(),
            key_struct_ptr,
            key_handle_ptr,
        );
    }
}

/// Destroys an owned listener key.
///
/// # Safety
/// `key` must be a live owned handle and must not be used after this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_listener_key_drop(key: iox2_listener_key_h) {
    key.assert_non_null();
    unsafe {
        let value = &mut *key.as_type();
        core::ptr::drop_in_place(value.value.as_option_mut());
        (value.deleter)(value);
    }
}

// END C API
