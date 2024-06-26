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

use crate::{
    iox2_node_h, iox2_node_name_h, iox2_node_storage_t, iox2_node_type_e, IntoCInt, IOX2_OK,
};

use iceoryx2::node::NodeCreationFailure;
use iceoryx2::prelude::*;
use iceoryx2_bb_elementary::static_assert::*;

use core::ffi::c_int;
use core::mem::{align_of, size_of, MaybeUninit};
use std::alloc::{alloc, dealloc, Layout};

// BEGIN types definition

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_node_creation_failure_e {
    INSUFFICIENT_PERMISSIONS = IOX2_OK as isize + 1,
    INTERNAL_ERROR,
}

impl IntoCInt for NodeCreationFailure {
    fn into_c_int(self) -> c_int {
        (match self {
            NodeCreationFailure::InsufficientPermissions => {
                iox2_node_creation_failure_e::INSUFFICIENT_PERMISSIONS
            }
            NodeCreationFailure::InternalError => iox2_node_creation_failure_e::INTERNAL_ERROR,
        }) as c_int
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of NodeBuilder
pub struct iox2_node_builder_storage_internal_t {
    internal: [u8; 18416], // magic number obtained with size_of::<NodeBuilder>()
}

#[repr(C)]
pub struct iox2_node_builder_storage_t {
    internal: iox2_node_builder_storage_internal_t,
    deleter: fn(*mut iox2_node_builder_storage_t),
}

/// The handle to use for the `iox2_node_builder_*` functions
pub type iox2_node_builder_h = *mut iox2_node_builder_storage_t;

impl iox2_node_builder_storage_t {
    const fn assert_storage_layout() {
        static_assert_ge::<
            { align_of::<iox2_node_builder_storage_internal_t>() },
            { align_of::<NodeBuilder>() },
        >();
        static_assert_ge::<
            { size_of::<iox2_node_builder_storage_internal_t>() },
            { size_of::<NodeBuilder>() },
        >();
    }

    fn node_builder_maybe_uninit(&mut self) -> &mut MaybeUninit<NodeBuilder> {
        iox2_node_builder_storage_t::assert_storage_layout();

        unsafe {
            &mut *(&mut self.internal as *mut iox2_node_builder_storage_internal_t)
                .cast::<MaybeUninit<NodeBuilder>>()
        }
    }
    unsafe fn node_builder_assume_init(&mut self) -> &mut NodeBuilder {
        self.node_builder_maybe_uninit().assume_init_mut()
    }

    fn alloc() -> *mut iox2_node_builder_storage_t {
        unsafe {
            alloc(Layout::new::<iox2_node_builder_storage_t>()) as *mut iox2_node_builder_storage_t
        }
    }
    fn dealloc(storage: *mut iox2_node_builder_storage_t) {
        unsafe {
            dealloc(
                storage as *mut _,
                Layout::new::<iox2_node_builder_storage_t>(),
            );
        }
    }
}

// END type definition

// BEGIN C API

/// Creates a builder for nodes
///
/// # Arguments
///
/// * `node_builder_storage` - Must be either a NULL pointer or a pointer to a valid [`iox2_node_builder_storage_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
///
/// # Returns
///
/// A [`iox2_node_builder_h`] handle to build the actual node.
///
/// # Safety
///
/// The same [`iox2_node_builder_storage_t`] cannot be used in subsequent calls to this function, unless [`iox2_node_builder_create`] was called before!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_builder_new(
    node_builder_storage: *mut iox2_node_builder_storage_t,
) -> iox2_node_builder_h {
    let mut handle = node_builder_storage;
    fn no_op(_storage: *mut iox2_node_builder_storage_t) {}
    let mut deleter: fn(*mut iox2_node_builder_storage_t) = no_op;
    if handle.is_null() {
        handle = iox2_node_builder_storage_t::alloc();
        deleter = iox2_node_builder_storage_t::dealloc;
    }
    debug_assert!(!handle.is_null());

    unsafe {
        (*handle).deleter = deleter;
    }

    unsafe {
        (*handle)
            .node_builder_maybe_uninit()
            .write(NodeBuilder::new());
    }

    handle
}

#[no_mangle]
pub extern "C" fn iox2_node_builder_set_name(
    node_builder_handle: iox2_node_builder_h,
    node_name_handle: iox2_node_name_h,
) -> c_int {
    debug_assert!(!node_builder_handle.is_null());
    debug_assert!(!node_name_handle.is_null());
    todo!() // TODO: [#210] implement

    // IOX2_OK
}

#[no_mangle]
pub extern "C" fn iox2_node_builder_set_config(node_builder_handle: iox2_node_builder_h) -> c_int {
    debug_assert!(!node_builder_handle.is_null());
    todo!() // TODO: [#210] implement

    // IOX2_OK
}

/// Creates a node and consumes the builder
///
/// # Arguments
///
/// * `node_builder_handle` - Must be a valid [`iox2_node_builder_h`] obtained by [`iox2_node_builder_new`].
/// * `node_storage` - Must be either a NULL pointer or a pointer to a valid [`iox2_node_storage_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `node_type` - The [`iox2_node_type_e`] for the node to be created.
/// * `node_handle_ptr` - An uninitialized or dangling [`iox2_node_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_node_creation_failure_e`] otherwise.
///
/// # Safety
///
/// The `node_builder_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// The corresponding [`iox2_node_builder_storage_t`] can be re-used with a call to [`iox2_node_builder_new`]!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_builder_create(
    node_builder_handle: iox2_node_builder_h,
    node_storage: *mut iox2_node_storage_t,
    node_type: iox2_node_type_e,
    node_handle_ptr: *mut iox2_node_h,
) -> c_int {
    debug_assert!(!node_builder_handle.is_null());
    debug_assert!(!node_handle_ptr.is_null());

    let node_builder = std::mem::take(unsafe { (*node_builder_handle).node_builder_assume_init() });
    unsafe {
        std::ptr::drop_in_place((*node_builder_handle).node_builder_assume_init() as *mut _);
        ((*node_builder_handle).deleter)(node_builder_handle);
    }

    let mut node_handle = node_storage;
    fn no_op(_storage: *mut iox2_node_storage_t) {}
    let mut deleter: fn(*mut iox2_node_storage_t) = no_op;
    if node_handle.is_null() {
        node_handle = iox2_node_storage_t::alloc();
        deleter = iox2_node_storage_t::dealloc;
    }
    debug_assert!(!node_handle.is_null());

    unsafe {
        (*node_handle).node_type = node_type;
        (*node_handle).deleter = deleter;
        *node_handle_ptr = node_handle;
    }

    match node_type {
        iox2_node_type_e::ZERO_COPY => match node_builder.create::<zero_copy::Service>() {
            Ok(node) => unsafe {
                (*node_handle).node_maybe_uninit().write(node);
            },
            Err(error) => {
                return error.into_c_int();
            }
        },
        iox2_node_type_e::PROCESS_LOCAL => match node_builder.create::<process_local::Service>() {
            Ok(node) => unsafe {
                (*node_handle).node_maybe_uninit().write(node);
            },
            Err(error) => {
                return error.into_c_int();
            }
        },
    }

    IOX2_OK
}

// END C API

#[cfg(test)]
mod test {
    use crate::*;
    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn assert_storage_sizes() {
        // all const functions; if it compiles, the storage size is sufficient
        const _STORAGE_LAYOUT_CHECK: () = iox2_node_builder_storage_t::assert_storage_layout();
    }

    #[test]
    fn basic_node_builder_api_test() {
        unsafe {
            let node_builder_handle = iox2_node_builder_new(std::ptr::null_mut());
            let mut node_handle: iox2_node_h = std::ptr::null_mut();
            let ret_val = iox2_node_builder_create(
                node_builder_handle,
                std::ptr::null_mut(),
                iox2_node_type_e::PROCESS_LOCAL,
                &mut node_handle as *mut iox2_node_h,
            );

            assert_that!(ret_val, eq(IOX2_OK));
            assert_that!(node_handle, ne(std::ptr::null_mut()));

            iox2_node_drop(node_handle);
        }
    }
}
