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

use core::ffi::c_int;

use crate::{
    api::IntoCInt, iox2_service_type_e, iox2_waitset_h, iox2_waitset_t, WaitSetUnion, IOX2_OK,
};

use super::{iox2_signal_handling_mode_e, AssertNonNullHandle, HandleToType};
use iceoryx2::prelude::WaitSetBuilder;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

#[repr(C)]
#[repr(align(1))] // alignment of Option<WaitSetBuilder>
pub struct iox2_waitset_builder_storage_t {
    internal: [u8; 1], // magic number obtained with size_of::<Option<WaitSetBuilder>>()
}

#[repr(C)]
#[iceoryx2_ffi(WaitSetBuilder)]
pub struct iox2_waitset_builder_t {
    pub value: iox2_waitset_builder_storage_t,
    deleter: fn(*mut iox2_waitset_builder_t),
}

pub struct iox2_waitset_builder_h_t;
/// The owning handle for `iox2_waitset_builder_t`. Passing the handle to an function transfers the ownership.
pub type iox2_waitset_builder_h = *mut iox2_waitset_builder_h_t;

/// The non-owning handle for `iox2_waitset_builder_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_waitset_builder_h_ref = *const iox2_waitset_builder_h;

// NOTE check the README.md for using opaque types with renaming
/// The immutable pointer to the underlying `NodeName`
pub type iox2_waitset_builder_ptr = *const WaitSetBuilder;
/// The mutable pointer to the underlying `NodeName`
pub type iox2_waitset_builder_ptr_mut = *mut WaitSetBuilder;

impl AssertNonNullHandle for iox2_waitset_builder_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_waitset_builder_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_waitset_builder_h {
    type Target = *mut iox2_waitset_builder_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_waitset_builder_h_ref {
    type Target = *mut iox2_waitset_builder_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition

// BEGIN C API
/// Creates a new [`iox2_waitset_builder_t`] to create a [`iox2_waitset_t`] with
/// [`iox2_waitset_builder_create()`]
///
/// # Safety
///
///  * `struct_ptr` must be either a valid pointer to uninitialized memory or `null`
///  * `handle_ptr` must point to a valid uninitialized memory location
///  * The acquire handle must be cleaned up with [`iox2_waitset_builder_drop()`].
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_builder_new(
    struct_ptr: *mut iox2_waitset_builder_t,
    handle_ptr: *mut iox2_waitset_builder_h,
) {
    debug_assert!(!handle_ptr.is_null());

    *handle_ptr = core::ptr::null_mut();

    let mut struct_ptr = struct_ptr;
    fn no_op(_: *mut iox2_waitset_builder_t) {}
    let mut deleter: fn(*mut iox2_waitset_builder_t) = no_op;
    if struct_ptr.is_null() {
        struct_ptr = iox2_waitset_builder_t::alloc();
        deleter = iox2_waitset_builder_t::dealloc;
    }
    debug_assert!(!struct_ptr.is_null());

    (*struct_ptr).deleter = deleter;
    (*struct_ptr).value.init(WaitSetBuilder::new());

    *handle_ptr = (*struct_ptr).as_handle();
}

/// Drops a [`iox2_waitset_builder_h`] and calls all corresponding cleanup functions.
///
/// # Safety
///
///  * `handle` must be acquired with [`iox2_waitset_builder_new()`]
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_builder_drop(handle: iox2_waitset_builder_h) {
    debug_assert!(!handle.is_null());

    let waitset_builder = &mut *handle.as_type();
    core::ptr::drop_in_place(waitset_builder.value.as_option_mut());
    (waitset_builder.deleter)(waitset_builder);
}

/// Creates a new [`iox2_waitset_t`].
///
/// # Returns
///
///  [`IOX2_OK`] on success otherwise
///  [`iox2_waitset_create_error_e`](crate::iox2_waitset_create_error_e).
///
/// # Safety
///
///  * `handle` must be acquired with [`iox2_waitset_builder_new()`] and valid
///  * `handle` is invalidated after a successful operation and cannot used again
///    with this function.
///  * `struct_ptr` must be either a valid pointer to uninitialized memory or `null`
///  * `handle_ptr` must point to a valid uninitialized memory location
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_builder_create(
    handle: iox2_waitset_builder_h,
    service_type: iox2_service_type_e,
    struct_ptr: *mut iox2_waitset_t,
    handle_ptr: *mut iox2_waitset_h,
) -> c_int {
    debug_assert!(!handle.is_null());

    let waitset_builder_struct = unsafe { &mut *handle.as_type() };
    let waitset_builder = waitset_builder_struct.take().unwrap();
    iox2_waitset_builder_drop(handle);

    fn no_op(_: *mut iox2_waitset_t) {}
    let mut deleter: fn(*mut iox2_waitset_t) = no_op;
    let mut struct_ptr = struct_ptr;
    *handle_ptr = core::ptr::null_mut();

    let mut alloc_memory = || {
        if struct_ptr.is_null() {
            struct_ptr = iox2_waitset_t::alloc();
            deleter = iox2_waitset_t::dealloc;
        }
        debug_assert!(!struct_ptr.is_null());

        (*struct_ptr).deleter = deleter;
    };

    match service_type {
        iox2_service_type_e::IPC => {
            let waitset = match waitset_builder.create::<crate::IpcService>() {
                Ok(waitset) => waitset,
                Err(e) => return e.into_c_int(),
            };

            alloc_memory();

            (*struct_ptr).init(service_type, WaitSetUnion::new_ipc(waitset), deleter);
        }
        iox2_service_type_e::LOCAL => {
            let waitset = match waitset_builder.create::<crate::LocalService>() {
                Ok(waitset) => waitset,
                Err(e) => return e.into_c_int(),
            };

            alloc_memory();

            (*struct_ptr).init(service_type, WaitSetUnion::new_local(waitset), deleter);
        }
    }

    *handle_ptr = (*struct_ptr).as_handle();
    IOX2_OK
}

/// Sets the [`iox2_signal_handling_mode_e`] for the [`iox2_waitset_h`].
///
/// # Arguments
///
/// * `waitset_builder_handle` - Must be a valid [`iox2_waitset_builder_h_ref`] obtained by [`iox2_waitset_builder_new`].
///
/// # Safety
///
/// * `waitset_builder_handle` must be a valid handle
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_builder_set_signal_handling_mode(
    waitset_builder_handle: iox2_waitset_builder_h_ref,
    signal_handling_mode: iox2_signal_handling_mode_e,
) {
    waitset_builder_handle.assert_non_null();

    let waitset_builder_struct = &mut *waitset_builder_handle.as_type();

    let waitset_builder = waitset_builder_struct.take().unwrap();
    let waitset_builder = waitset_builder.signal_handling_mode(signal_handling_mode.into());
    waitset_builder_struct.set(waitset_builder);
}

// END C API
