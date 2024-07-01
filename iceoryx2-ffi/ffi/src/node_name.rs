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

use iceoryx2::prelude::*;
use iceoryx2_bb_elementary::static_assert::*;

use core::ffi::{c_char, c_int};
use core::mem::{align_of, size_of};

// BEGIN type definition

#[repr(C)]
#[repr(align(8))] // alignment of NodeName
pub struct iox2_node_name_storage_internal_t {
    internal: [u8; 24], // magic number obtained with size_of::<NodeName>()
}

#[repr(C)]
pub struct iox2_node_name_storage_t {
    internal: iox2_node_name_storage_internal_t,
    deleter: fn(*mut iox2_node_name_storage_t),
}

/// The handle to use for the `iox2_node_name_*` functions
pub type iox2_node_name_h = *mut iox2_node_name_storage_t;

/// The immutable handle to the underlying `NodeName`
pub type iox2_node_name_t = *const iox2_node_name_storage_internal_t;

impl iox2_node_name_storage_t {
    const fn _assert_storage_layout() {
        static_assert_ge::<
            { align_of::<iox2_node_name_storage_internal_t>() },
            { align_of::<NodeName>() },
        >();
        static_assert_ge::<
            { size_of::<iox2_node_name_storage_internal_t>() },
            { size_of::<NodeName>() },
        >();
    }
}

// END type definition

// BEGIN C API

/// This function create a new node name!
///
/// # Arguments
///
/// * `node_name_storage` - Must be either a NULL pointer or a pointer to a valid [`iox2_node_name_storage_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `node_name` - Must be valid node name string.
/// * `node_name_len` - The length of the node name string, not including a null termination.
/// * `node_name_handle_ptr` - An uninitialized or dangling [`iox2_node_name_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_semantic_string_error_e`](crate::iox2_semantic_string_error_e) otherwise.
///
/// # Safety
///
/// Terminates if `node_name` or `node_name_handle_ptr` is a NULL pointer!
/// It is undefined behavior to pass a `node_name_len` which is larger than the actual length of `node_name`!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_name_new(
    _node_name_storage: *mut iox2_node_name_storage_t,
    node_name: *const c_char,
    _node_name_len: c_int,
    node_name_handle_ptr: *mut iox2_node_name_h,
) -> c_int {
    assert!(!node_name.is_null());
    assert!(!node_name_handle_ptr.is_null());

    unimplemented!() // TODO: [#210] implement

    // IOX2_OK
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
/// The `node_name_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// The corresponding [`iox2_node_name_storage_t`] can be re-used with a call to [`iox2_node_name_new`]!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_name_drop(node_name_handle: iox2_node_name_h) {
    assert!(!node_name_handle.is_null());

    unimplemented!() // TODO: [#210] implement
}

// END C API

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn assert_storage_size() {
        // all const functions; if it compiles, the storage size is sufficient
        const _STORAGE_LAYOUT_CHECK: () = iox2_node_name_storage_t::_assert_storage_layout();
    }
}
