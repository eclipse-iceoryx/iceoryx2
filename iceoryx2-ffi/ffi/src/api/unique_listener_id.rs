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

use iceoryx2::port::port_identifiers::UniqueListenerId;
use iceoryx2_bb_elementary::static_assert::static_assert_ge;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use super::HandleToType;

// BEGIN types definition

/// The system-wide unique id of a `iox2_listener_t`.
#[repr(C)]
#[repr(align(4))] // core::mem::align_of::<UniqueListenerId>()
pub struct iox2_unique_listener_id_storage_t {
    internal: [u8; 20], // core::mem::size_of::<Option<UniqueListenerId>>()
}

#[repr(C)]
#[iceoryx2_ffi(UniqueListenerId)]
pub struct iox2_unique_listener_id_t {
    pub value: iox2_unique_listener_id_storage_t,
    pub(super) deleter: fn(*mut iox2_unique_listener_id_t),
}

impl iox2_unique_listener_id_t {
    pub(super) fn init(
        &mut self,
        value: UniqueListenerId,
        deleter: fn(*mut iox2_unique_listener_id_t),
    ) {
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_unique_listener_id_h_t;
/// The owning handle for [`iox2_unique_listener_id_t`]. Passing the handle to an function transfers the ownership.
pub type iox2_unique_listener_id_h = *mut iox2_unique_listener_id_h_t;

pub struct iox2_unique_listener_id_h_ref_t;
/// The non-owning handle for [`iox2_unique_listener_id_t`]. Passing the handle to an function does not transfers the ownership.
pub type iox2_unique_listener_id_h_ref = *mut iox2_unique_listener_id_h_ref_t;

impl HandleToType for iox2_unique_listener_id_h {
    type Target = *mut iox2_unique_listener_id_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_unique_listener_id_h_ref {
    type Target = *mut iox2_unique_listener_id_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

// END types definition

// BEGIN C API

/// This function needs to be called to destroy the unique listener id!
///
/// # Arguments
///
/// * `handle` - A valid [`iox2_unique_listener_id_h`]
///
/// # Safety
///
/// * The `handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
#[no_mangle]
pub unsafe extern "C" fn iox2_unique_listener_id_drop(handle: iox2_unique_listener_id_h) {
    debug_assert!(!handle.is_null());

    let h = &mut *handle.as_type();
    core::ptr::drop_in_place(h.value.as_option_mut());
    (h.deleter)(h);
}

/// This function casts an owning [`iox2_unique_listener_id_h`] into a non-owning
/// [`iox2_unique_listener_id_h_ref`]
///
/// Returns a [`iox2_unique_listener_id_h_ref`]
///
/// # Safety
///
/// * The `listener_handle` must be a valid handle.
/// * The `listener_handle` is still valid after the call to this function.
#[no_mangle]
pub unsafe extern "C" fn iox2_cast_unique_listener_id_h_ref(
    handle: iox2_unique_listener_id_h,
) -> iox2_unique_listener_id_h_ref {
    debug_assert!(!handle.is_null());

    (*handle.as_type()).as_h_refandle() as *mut _ as _
}

/// Checks two [`iox2_unique_listener_id_t`] for equality.
///
/// # Safety
///
/// * `lhs` - Must be a valid [`iox2_unique_listener_id_h_ref`]
/// * `rhs` - Must be a valid [`iox2_unique_listener_id_h_ref`]
#[no_mangle]
pub unsafe extern "C" fn iox2_unique_listener_id_eq(
    lhs: iox2_unique_listener_id_h_ref,
    rhs: iox2_unique_listener_id_h_ref,
) -> bool {
    debug_assert!(!lhs.is_null());
    debug_assert!(!rhs.is_null());

    let lhs = &mut *lhs.as_type();
    let rhs = &mut *rhs.as_type();

    lhs.value.as_ref() == rhs.value.as_ref()
}

/// Checks the ordering of two [`iox2_unique_listener_id_t`].
///
/// # Safety
///
/// * `lhs` - Must be a valid [`iox2_unique_listener_id_h_ref`]
/// * `rhs` - Must be a valid [`iox2_unique_listener_id_h_ref`]
#[no_mangle]
pub unsafe extern "C" fn iox2_unique_listener_id_less(
    lhs: iox2_unique_listener_id_h_ref,
    rhs: iox2_unique_listener_id_h_ref,
) -> bool {
    debug_assert!(!lhs.is_null());
    debug_assert!(!rhs.is_null());

    let lhs = &mut *lhs.as_type();
    let rhs = &mut *rhs.as_type();

    lhs.value.as_ref() < rhs.value.as_ref()
}

// END C API
