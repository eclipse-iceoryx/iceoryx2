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

use crate::api::{
    iox2_semantic_string_error_e, AssertNonNullHandle, HandleToType, IntoCInt, IOX2_OK,
};
use crate::c_size_t;

use iceoryx2::prelude::*;
use iceoryx2::service::service_name::ServiceNameError;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_bb_elementary_traits::AsCStr;
use iceoryx2_ffi_macros::{iceoryx2_ffi, CStrRepr};

use core::ffi::{c_char, c_int};
use core::{slice, str};

// BEGIN type definition

#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_service_name_error_e {
    INVALID_CONTENT = IOX2_OK as isize + 1,
    EXCEEDS_MAXIMUM_LENGTH,
}

impl IntoCInt for ServiceNameError {
    fn into_c_int(self) -> c_int {
        (match self {
            ServiceNameError::InvalidContent => iox2_service_name_error_e::INVALID_CONTENT,
            ServiceNameError::ExceedsMaximumLength => {
                iox2_service_name_error_e::EXCEEDS_MAXIMUM_LENGTH
            }
        }) as c_int
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<ServiceName>
pub struct iox2_service_name_storage_t {
    internal: [u8; 272], // magic number obtained with size_of::<Option<ServiceName>>()
}

#[repr(C)]
#[iceoryx2_ffi(ServiceName)]
pub struct iox2_service_name_t {
    pub value: iox2_service_name_storage_t,
    deleter: fn(*mut iox2_service_name_t),
}

pub struct iox2_service_name_h_t;
/// The owning handle for `iox2_service_name_t`. Passing the handle to an function transfers the ownership.
pub type iox2_service_name_h = *mut iox2_service_name_h_t;

/// The non-owning handle for `iox2_service_name_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_service_name_h_ref = *const iox2_service_name_h;

// NOTE check the README.md for using opaque types with renaming
/// The immutable pointer to the underlying `ServiceName`
pub type iox2_service_name_ptr = *const ServiceName;
/// The mutable pointer to the underlying `ServiceName`
pub type iox2_service_name_ptr_mut = *mut ServiceName;

impl AssertNonNullHandle for iox2_service_name_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_service_name_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_service_name_h {
    type Target = *mut iox2_service_name_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_service_name_h_ref {
    type Target = *mut iox2_service_name_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition

// BEGIN C API

/// This function create a new node name!
///
/// # Arguments
///
/// * `service_name_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_service_name_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `service_name_str` - Must be valid node name string.
/// * `service_name_len` - The length of the node name string, not including a null termination.
/// * `service_name_handle_ptr` - An uninitialized or dangling [`iox2_service_name_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_semantic_string_error_e`](crate::iox2_semantic_string_error_e) otherwise.
///
/// # Safety
///
/// * Terminates if `service_name_str` or `service_name_handle_ptr` is a NULL pointer!
/// * It is undefined behavior to pass a `service_name_len` which is larger than the actual length of `service_name_str`!
#[no_mangle]
pub unsafe extern "C" fn iox2_service_name_new(
    service_name_struct_ptr: *mut iox2_service_name_t,
    service_name_str: *const c_char,
    service_name_len: c_size_t,
    service_name_handle_ptr: *mut iox2_service_name_h,
) -> c_int {
    debug_assert!(!service_name_str.is_null());
    debug_assert!(!service_name_handle_ptr.is_null());

    *service_name_handle_ptr = core::ptr::null_mut();

    let mut service_name_struct_ptr = service_name_struct_ptr;
    fn no_op(_: *mut iox2_service_name_t) {}
    let mut deleter: fn(*mut iox2_service_name_t) = no_op;
    if service_name_struct_ptr.is_null() {
        service_name_struct_ptr = iox2_service_name_t::alloc();
        deleter = iox2_service_name_t::dealloc;
    }
    debug_assert!(!service_name_struct_ptr.is_null());

    unsafe {
        (*service_name_struct_ptr).deleter = deleter;
    }

    let service_name = slice::from_raw_parts(service_name_str as _, service_name_len as _);

    let service_name = if let Ok(service_name) = str::from_utf8(service_name) {
        service_name
    } else {
        deleter(service_name_struct_ptr);
        return iox2_semantic_string_error_e::INVALID_CONTENT as c_int;
    };

    let service_name = match ServiceName::new(service_name) {
        Ok(service_name) => service_name,
        Err(e) => {
            deleter(service_name_struct_ptr);
            return e.into_c_int();
        }
    };

    unsafe {
        (*service_name_struct_ptr).value.init(service_name);
    }

    *service_name_handle_ptr = (*service_name_struct_ptr).as_handle();

    IOX2_OK
}

/// This function casts a [`iox2_service_name_h`] into a [`iox2_service_name_ptr`]
///
/// # Arguments
///
/// * `service_name_handle` obtained by [`iox2_service_name_new`]
///
/// Returns a [`iox2_service_name_ptr`]
///
/// # Safety
///
/// * The `service_name_handle` must be a valid handle.
/// * The `service_name_handle` is still valid after the call to this function.
#[no_mangle]
pub unsafe extern "C" fn iox2_cast_service_name_ptr(
    service_name_handle: iox2_service_name_h,
) -> iox2_service_name_ptr {
    debug_assert!(!service_name_handle.is_null());

    (*service_name_handle.as_type()).value.as_ref()
}

/// This function gives access to the node name as a non-zero-terminated char array
///
/// # Arguments
///
/// * `service_name_ptr` obtained by e.g. [`iox2_cast_service_name_ptr`] or a function returning a [`iox2_service_name_ptr`]
/// * `service_name_len` must be used to get the length of the char array
///
/// Returns non-zero-terminated char array
///
/// # Safety
///
/// * The `service_name_ptr` must be a valid pointer to a node name.
/// * The `service_name_len` must be a valid pointer to a size_t.
#[no_mangle]
pub unsafe extern "C" fn iox2_service_name_as_chars(
    service_name_ptr: iox2_service_name_ptr,
    service_name_len: *mut c_size_t,
) -> *const c_char {
    debug_assert!(!service_name_ptr.is_null());
    debug_assert!(!service_name_len.is_null());

    let service_name = &*service_name_ptr;

    if !service_name_len.is_null() {
        unsafe {
            *service_name_len = service_name.len() as _;
        }
    }

    service_name.as_str().as_ptr() as _
}

/// This function needs to be called to destroy the node name!
///
/// In general, this function is not required to call, since [`iox2_node_builder_set_name`](crate::iox2_node_builder_set_name) will consume the [`iox2_service_name_h`] handle.
///
/// # Arguments
///
/// * `service_name_handle` - A valid [`iox2_service_name_h`]
///
/// # Safety
///
/// * The `service_name_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_service_name_t`] can be re-used with a call to [`iox2_service_name_new`]!
#[no_mangle]
pub unsafe extern "C" fn iox2_service_name_drop(service_name_handle: iox2_service_name_h) {
    debug_assert!(!service_name_handle.is_null());

    let service_name = &mut *service_name_handle.as_type();

    core::ptr::drop_in_place(service_name.value.as_option_mut());
    (service_name.deleter)(service_name);
}

// END C API
