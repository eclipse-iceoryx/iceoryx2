// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use iceoryx2::port::port_identifiers::UniqueNotifierId;
use iceoryx2_bb_elementary::static_assert::static_assert_ge;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use crate::api::{AssertNonNullHandle, HandleToType};

// BEGIN types definition

/// The system-wide unique id of a `iox2_notifier_t`.
#[repr(C)]
#[repr(align(4))] // core::mem::align_of::<UniqueNotifierId>()
pub struct iox2_unique_notifier_id_storage_t {
    internal: [u8; 20], // core::mem::size_of::<Option<UniqueNotifierId>>()
}

#[repr(C)]
#[iceoryx2_ffi(UniqueNotifierId)]
pub struct iox2_unique_notifier_id_t {
    pub value: iox2_unique_notifier_id_storage_t,
    pub(super) deleter: fn(*mut iox2_unique_notifier_id_t),
}

impl iox2_unique_notifier_id_t {
    pub(super) fn init(
        &mut self,
        value: UniqueNotifierId,
        deleter: fn(*mut iox2_unique_notifier_id_t),
    ) {
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_unique_notifier_id_h_t;
/// The owning handle for [`iox2_unique_notifier_id_t`]. Passing the handle to an function transfers the ownership.
pub type iox2_unique_notifier_id_h = *mut iox2_unique_notifier_id_h_t;
/// The non-owning handle for [`iox2_unique_notifier_id_t`]. Passing the handle to an function does not transfers the ownership.
pub type iox2_unique_notifier_id_h_ref = *const iox2_unique_notifier_id_h;

impl AssertNonNullHandle for iox2_unique_notifier_id_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_unique_notifier_id_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_unique_notifier_id_h {
    type Target = *mut iox2_unique_notifier_id_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_unique_notifier_id_h_ref {
    type Target = *mut iox2_unique_notifier_id_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END types definition

// BEGIN C API

/// Retrieves the value of a unique notifier ID.
///
/// # Arguments
///
/// * `handle` - A valid [`iox2_unique_notifier_id_h`]
/// * `id_ptr` - Pointer to a buffer where the ID value will be written
/// * `id_length` - The length of the buffer pointed to by `id_ptr`
///
/// # Safety
///
/// * `handle` must be a valid, non-null pointer
/// * `id_ptr` must be a valid, non-null pointer to a buffer of at least `id_length` bytes
/// * `id_length` must be large enough to hold the ID value
#[no_mangle]
unsafe extern "C" fn iox2_unique_notifier_id_value(
    handle: iox2_unique_notifier_id_h,
    id_ptr: *mut u8,
    id_length: usize,
) {
    debug_assert!(!id_ptr.is_null());
    handle.assert_non_null();

    let h = &mut *handle.as_type();

    if let Some(Some(id)) = (h.value.internal.as_ptr() as *const Option<UniqueNotifierId>).as_ref()
    {
        let bytes = id.value().to_ne_bytes();
        debug_assert!(bytes.len() <= id_length, "id_length is too small");

        unsafe {
            core::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                id_ptr,
                core::cmp::min(bytes.len(), id_length),
            );
        }
    }
}

/// This function needs to be called to destroy the unique notifier id!
///
/// # Arguments
///
/// * `handle` - A valid [`iox2_unique_notifier_id_h`]
///
/// # Safety
///
/// * The `handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
#[no_mangle]
pub unsafe extern "C" fn iox2_unique_notifier_id_drop(handle: iox2_unique_notifier_id_h) {
    debug_assert!(!handle.is_null());

    let h = &mut *handle.as_type();
    core::ptr::drop_in_place(h.value.as_option_mut());
    (h.deleter)(h);
}

/// Checks two [`iox2_unique_notifier_id_t`] for equality.
///
/// # Safety
///
/// * `lhs` - Must be a valid [`iox2_unique_notifier_id_h_ref`]
/// * `rhs` - Must be a valid [`iox2_unique_notifier_id_h_ref`]
#[no_mangle]
pub unsafe extern "C" fn iox2_unique_notifier_id_eq(
    lhs: iox2_unique_notifier_id_h_ref,
    rhs: iox2_unique_notifier_id_h_ref,
) -> bool {
    lhs.assert_non_null();
    rhs.assert_non_null();

    let lhs = &mut *lhs.as_type();
    let rhs = &mut *rhs.as_type();

    lhs.value.as_ref() == rhs.value.as_ref()
}

/// Checks the ordering of two [`iox2_unique_notifier_id_t`].
///
/// # Safety
///
/// * `lhs` - Must be a valid [`iox2_unique_notifier_id_h_ref`]
/// * `rhs` - Must be a valid [`iox2_unique_notifier_id_h_ref`]
#[no_mangle]
pub unsafe extern "C" fn iox2_unique_notifier_id_less(
    lhs: iox2_unique_notifier_id_h_ref,
    rhs: iox2_unique_notifier_id_h_ref,
) -> bool {
    lhs.assert_non_null();
    rhs.assert_non_null();

    let lhs = &mut *lhs.as_type();
    let rhs = &mut *rhs.as_type();

    lhs.value.as_ref() < rhs.value.as_ref()
}

// END C API
