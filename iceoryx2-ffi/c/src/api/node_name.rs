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
    c_size_t, iox2_semantic_string_error_e, AssertNonNullHandle, HandleToType, IntoCInt, IOX2_OK,
};

use iceoryx2::prelude::*;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::ffi::{c_char, c_int};
use core::{slice, str};

// BEGIN type definition

#[repr(C)]
#[repr(align(8))] // alignment of Option<NodeName>
pub struct iox2_node_name_storage_t {
    internal: [u8; 152], // magic number obtained with size_of::<Option<NodeName>>()
}

#[repr(C)]
#[iceoryx2_ffi(NodeName)]
pub struct iox2_node_name_t {
    pub value: iox2_node_name_storage_t,
    deleter: fn(*mut iox2_node_name_t),
}

pub struct iox2_node_name_h_t;
/// The owning handle for `iox2_node_name_t`. Passing the handle to an function transfers the ownership.
pub type iox2_node_name_h = *mut iox2_node_name_h_t;

/// The non-owning handle for `iox2_node_name_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_node_name_h_ref = *const iox2_node_name_h;

// NOTE check the README.md for using opaque types with renaming
/// The immutable pointer to the underlying `NodeName`
pub type iox2_node_name_ptr = *const NodeName;
/// The mutable pointer to the underlying `NodeName`
pub type iox2_node_name_ptr_mut = *mut NodeName;

impl AssertNonNullHandle for iox2_node_name_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_node_name_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_node_name_h {
    type Target = *mut iox2_node_name_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_node_name_h_ref {
    type Target = *mut iox2_node_name_t;

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
/// * `node_name_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_node_name_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `node_name_str` - Must be valid node name string.
/// * `node_name_len` - The length of the node name string, not including a null termination.
/// * `node_name_handle_ptr` - An uninitialized or dangling [`iox2_node_name_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_semantic_string_error_e`](crate::iox2_semantic_string_error_e) otherwise.
///
/// # Safety
///
/// * Terminates if `node_name_str` or `node_name_handle_ptr` is a NULL pointer!
/// * It is undefined behavior to pass a `node_name_len` which is larger than the actual length of `node_name_str`!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_name_new(
    node_name_struct_ptr: *mut iox2_node_name_t,
    node_name_str: *const c_char,
    node_name_len: c_size_t,
    node_name_handle_ptr: *mut iox2_node_name_h,
) -> c_int {
    debug_assert!(!node_name_str.is_null());
    debug_assert!(!node_name_handle_ptr.is_null());

    *node_name_handle_ptr = core::ptr::null_mut();

    let mut node_name_struct_ptr = node_name_struct_ptr;
    fn no_op(_: *mut iox2_node_name_t) {}
    let mut deleter: fn(*mut iox2_node_name_t) = no_op;
    if node_name_struct_ptr.is_null() {
        node_name_struct_ptr = iox2_node_name_t::alloc();
        deleter = iox2_node_name_t::dealloc;
    }
    debug_assert!(!node_name_struct_ptr.is_null());

    unsafe {
        (*node_name_struct_ptr).deleter = deleter;
    }

    let node_name = slice::from_raw_parts(node_name_str as _, node_name_len as _);

    let node_name = if let Ok(node_name) = str::from_utf8(node_name) {
        node_name
    } else {
        deleter(node_name_struct_ptr);
        return iox2_semantic_string_error_e::INVALID_CONTENT as c_int;
    };

    let node_name = match NodeName::new(node_name) {
        Ok(node_name) => node_name,
        Err(e) => {
            deleter(node_name_struct_ptr);
            return e.into_c_int();
        }
    };

    unsafe {
        (*node_name_struct_ptr).value.init(node_name);
    }

    *node_name_handle_ptr = (*node_name_struct_ptr).as_handle();

    IOX2_OK
}

/// This function casts a [`iox2_node_name_h`] into a [`iox2_node_name_ptr`]
///
/// # Arguments
///
/// * `node_name_handle` obtained by [`iox2_node_name_new`]
///
/// Returns a [`iox2_node_name_ptr`]
///
/// # Safety
///
/// * The `node_name_handle` must be a valid handle.
/// * The `node_name_handle` is still valid after the call to this function.
#[no_mangle]
pub unsafe extern "C" fn iox2_cast_node_name_ptr(
    node_name_handle: iox2_node_name_h,
) -> iox2_node_name_ptr {
    debug_assert!(!node_name_handle.is_null());

    (*node_name_handle.as_type()).value.as_ref()
}

/// This function gives access to the node name as a non-zero-terminated char array
///
/// # Arguments
///
/// * `node_name_ptr` obtained by e.g. [`iox2_cast_node_name_ptr`] or a function returning a [`iox2_node_name_ptr`]
/// * `node_name_len` must be used to get the length of the char array
///
/// Returns a non-zero-terminated char array
///
/// # Safety
///
/// * The `node_name_ptr` must be a valid pointer to a node name.
/// * The `node_name_len` must be a valid pointer to a size_t.
#[no_mangle]
pub unsafe extern "C" fn iox2_node_name_as_chars(
    node_name_ptr: iox2_node_name_ptr,
    node_name_len: *mut c_size_t,
) -> *const c_char {
    debug_assert!(!node_name_ptr.is_null());
    debug_assert!(!node_name_len.is_null());

    let node_name = &*node_name_ptr;

    if !node_name_len.is_null() {
        unsafe {
            *node_name_len = node_name.len() as _;
        }
    }

    node_name.as_str().as_ptr() as _
}

/// This function needs to be called to destroy the node name!
///
/// In general, this function is not required to call, since [`iox2_node_builder_set_name`](crate::iox2_node_builder_set_name) will consume the [`iox2_node_name_h`] handle.
///
/// # Arguments
///
/// * `node_name_handle` - A valid [`iox2_node_name_h`]
///
/// # Safety
///
/// * The `node_name_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_node_name_t`] can be re-used with a call to [`iox2_node_name_new`]!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_name_drop(node_name_handle: iox2_node_name_h) {
    debug_assert!(!node_name_handle.is_null());

    let node_name = &mut *node_name_handle.as_type();

    core::ptr::drop_in_place(node_name.value.as_option_mut());
    (node_name.deleter)(node_name);
}

// END C API
