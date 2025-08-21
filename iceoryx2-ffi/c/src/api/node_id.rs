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

use crate::api::{AssertNonNullHandle, HandleToType};

use iceoryx2::node::NodeId;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

// BEGIN type definition

#[repr(C)]
#[repr(align(8))] // alignment of Option<NodeId>
pub struct iox2_node_id_storage_t {
    internal: [u8; 20], // magic number obtained with size_of::<Option<NodeId>>()
}

#[repr(C)]
#[iceoryx2_ffi(NodeId)]
pub struct iox2_node_id_t {
    pub value: iox2_node_id_storage_t,
    deleter: fn(*mut iox2_node_id_t),
}

pub struct iox2_node_id_h_t;
/// The owning handle for `iox2_node_id_t`. Passing the handle to an function transfers the ownership.
pub type iox2_node_id_h = *mut iox2_node_id_h_t;

/// The non-owning handle for `iox2_node_id_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_node_id_h_ref = *const iox2_node_id_h;

// NOTE check the README.md for using opaque types with renaming
/// The immutable pointer to the underlying `NodeId`
pub type iox2_node_id_ptr = *const NodeId;
/// The mutable pointer to the underlying `NodeId`
pub type iox2_node_id_ptr_mut = *mut NodeId;

impl AssertNonNullHandle for iox2_node_id_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_node_id_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_node_id_h {
    type Target = *mut iox2_node_id_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_node_id_h_ref {
    type Target = *mut iox2_node_id_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition

// BEGIN C API

/// Creates a new [`iox2_node_id_h`] by cloning a [`iox2_node_id_ptr`].
///
/// # Safety
///
/// * `node_id_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_node_id_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
/// * `node_id_ptr` - Must be a valid [`iox2_node_id_ptr`]
/// * `node_id_handle_ptr` - Must point to a valid [`iox2_node_id_h`].
#[no_mangle]
pub unsafe extern "C" fn iox2_node_id_clone_from_ptr(
    node_id_struct_ptr: *mut iox2_node_id_t,
    node_id_ptr: iox2_node_id_ptr,
    node_id_handle_ptr: *mut iox2_node_id_h,
) {
    debug_assert!(!node_id_handle_ptr.is_null());
    debug_assert!(!node_id_ptr.is_null());

    *node_id_handle_ptr = core::ptr::null_mut();

    let mut node_id_struct_ptr = node_id_struct_ptr;
    fn no_op(_: *mut iox2_node_id_t) {}
    let mut deleter: fn(*mut iox2_node_id_t) = no_op;
    if node_id_struct_ptr.is_null() {
        node_id_struct_ptr = iox2_node_id_t::alloc();
        deleter = iox2_node_id_t::dealloc;
    }
    debug_assert!(!node_id_struct_ptr.is_null());

    unsafe {
        (*node_id_struct_ptr).deleter = deleter;
    }

    (*node_id_struct_ptr).value.init(*node_id_ptr);
    *node_id_handle_ptr = (*node_id_struct_ptr).as_handle();
}

/// Creates a new [`iox2_node_id_h`] by cloning a [`iox2_node_id_h_ref`].
///
/// # Safety
///
/// * `node_id_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_node_id_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
/// * `node_id_handle` - Must be a valid [`iox2_node_id_h_ref`]
/// * `node_id_handle_ptr` - Must point to a valid [`iox2_node_id_h`].
#[no_mangle]
pub unsafe extern "C" fn iox2_node_id_clone_from_handle(
    node_id_struct_ptr: *mut iox2_node_id_t,
    node_id_handle: iox2_node_id_h_ref,
    node_id_handle_ptr: *mut iox2_node_id_h,
) {
    node_id_handle.assert_non_null();
    debug_assert!(!node_id_handle_ptr.is_null());

    let node_id = &mut *node_id_handle.as_type();
    let node_id_ptr = node_id.value.as_ref();

    iox2_node_id_clone_from_ptr(node_id_struct_ptr, node_id_ptr, node_id_handle_ptr);
}

/// Returns the high bits of the underlying value of the [`iox2_node_id_h`].
///
/// # Safety
///
/// * `node_id_handle` - Must be a valid [`iox2_node_id_h_ref`]
#[no_mangle]
pub unsafe extern "C" fn iox2_node_id_value_high(node_id_handle: iox2_node_id_h_ref) -> u64 {
    node_id_handle.assert_non_null();

    let node_id = &mut *node_id_handle.as_type();
    (node_id.value.as_ref().value() >> 64) as u64
}

/// Returns the low bits of the underlying value of the [`iox2_node_id_h`].
///
/// # Safety
///
/// * `node_id_handle` - Must be a valid [`iox2_node_id_h_ref`]
#[no_mangle]
pub unsafe extern "C" fn iox2_node_id_value_low(node_id_handle: iox2_node_id_h_ref) -> u64 {
    node_id_handle.assert_non_null();

    let node_id = &mut *node_id_handle.as_type();
    node_id.value.as_ref().value() as u64
}

/// Returns the process id of the [`iox2_node_id_h`].
///
/// # Safety
///
/// * `node_id_handle` - Must be a valid [`iox2_node_id_h_ref`]
#[no_mangle]
pub unsafe extern "C" fn iox2_node_id_pid(node_id_handle: iox2_node_id_h_ref) -> i32 {
    node_id_handle.assert_non_null();

    let node_id = &mut *node_id_handle.as_type();
    node_id.value.as_ref().pid().value() as _
}

/// Returns the creation time of the [`iox2_node_id_h`].
///
/// # Safety
///
/// * `node_id_handle` - Must be a valid [`iox2_node_id_h_ref`]
/// * `seconds` - Must point to a valid memory location
/// * `nanoseconds` - Must point to a valid memory location
#[no_mangle]
pub unsafe extern "C" fn iox2_node_id_creation_time(
    node_id_handle: iox2_node_id_h_ref,
    seconds: *mut u64,
    nanoseconds: *mut u32,
) {
    node_id_handle.assert_non_null();
    debug_assert!(!seconds.is_null());
    debug_assert!(!nanoseconds.is_null());

    let node_id = &mut *node_id_handle.as_type();
    *seconds = node_id.value.as_ref().creation_time().seconds();
    *nanoseconds = node_id.value.as_ref().creation_time().nanoseconds();
}

/// Takes ownership of the handle to delete and remove the underlying resources of a
/// [`iox2_node_id_h`].
///
/// # Safety
///
/// * `node_id_handle` - Must be a valid [`iox2_node_id_h`]
#[no_mangle]
pub unsafe extern "C" fn iox2_node_id_drop(node_id_handle: iox2_node_id_h) {
    debug_assert!(!node_id_handle.is_null());

    let node_id = &mut *node_id_handle.as_type();

    core::ptr::drop_in_place(node_id.value.as_option_mut());
    (node_id.deleter)(node_id);
}

// END C API
