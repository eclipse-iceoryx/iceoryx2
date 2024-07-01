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

use super::{IntoCInt, IOX2_OK};

use iceoryx2::prelude::*;
use iceoryx2::{node::NodeCreationFailure, service};
use iceoryx2_bb_elementary::math::max;
use iceoryx2_bb_elementary::static_assert::*;

use core::ffi::c_int;
use core::mem::{align_of, size_of, MaybeUninit};
use std::alloc::{alloc, dealloc, Layout};

// TODO: [#210] Add structs iox2_node_name_storage_internal_t, iox2_node_name_storage_t and iox2_node_name_h
// TODO: [#210] Add API iox2_node_name_new and iox2_node_name_set

// BEGIN iox2_node_builder_* types

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_node_creation_failure_e {
    INSUFFICIENT_PERMISSIONS = 1, // start at 1 since IOX2_OK is already 0
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
        static_assert_gt_or_equal::<
            { align_of::<iox2_node_builder_storage_internal_t>() },
            { align_of::<NodeBuilder>() },
        >();
        static_assert_gt_or_equal::<
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

// END iox2_node_builder_* types

// BEGIN iox2_builder_* types

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_node_type_e {
    PROCESS_LOCAL,
    ZERO_COPY,
}

#[repr(C)]
#[repr(align(8))] // magic number; the larger one of align_of::<Node<zero_copy::Service>>() and align_of::<Node<process_local::Service>>()
pub struct iox2_node_storage_internal_t {
    internal: [u8; 8], // magic number; the larger one of size_of::<Node<zero_copy::Service>>() and size_of::<Node<process_local::Service>>()
}

#[repr(C)]
pub struct iox2_node_storage_t {
    node_type: iox2_node_type_e,
    internal: iox2_node_storage_internal_t,
    deleter: fn(*mut iox2_node_storage_t),
}

/// The handle to use for the `iox2_node_*` functions
pub type iox2_node_h = *mut iox2_node_storage_t;

impl iox2_node_storage_t {
    const fn assert_storage_layout() {
        const MAX_NODE_ALIGNMENT: usize = max(
            align_of::<Node<zero_copy::Service>>(),
            align_of::<Node<process_local::Service>>(),
        );
        const MAX_NODE_SIZE: usize = max(
            size_of::<Node<zero_copy::Service>>(),
            size_of::<Node<process_local::Service>>(),
        );
        static_assert_gt_or_equal::<
            { align_of::<iox2_node_storage_internal_t>() },
            { MAX_NODE_ALIGNMENT },
        >();
        static_assert_gt_or_equal::<{ size_of::<iox2_node_storage_internal_t>() }, { MAX_NODE_SIZE }>(
        );
    }

    fn node_maybe_uninit<Service: service::Service>(&mut self) -> &mut MaybeUninit<Node<Service>> {
        iox2_node_storage_t::assert_storage_layout();

        unsafe {
            &mut *(&mut self.internal as *mut iox2_node_storage_internal_t)
                .cast::<MaybeUninit<Node<Service>>>()
        }
    }

    unsafe fn node_assume_init<Service: service::Service>(&mut self) -> &mut Node<Service> {
        self.node_maybe_uninit().assume_init_mut()
    }

    fn alloc() -> *mut iox2_node_storage_t {
        unsafe { alloc(Layout::new::<iox2_node_storage_t>()) as *mut iox2_node_storage_t }
    }
    fn dealloc(storage: *mut iox2_node_storage_t) {
        unsafe {
            dealloc(storage as *mut _, Layout::new::<iox2_node_storage_t>());
        }
    }
}

// END iox2_node_* types

// BEGIN iox2_node_builder_* C API

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
    assert!(!handle.is_null());

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

// TODO: [#210] add iox2_node_builder_set_name and iox2_node_builder_set_cofig

/// Creates a node and consumes the builder
///
/// # Arguments
///
/// * `node_builder_handle` - Must be a valid [`iox2_node_builder_h`] obtained by [`iox2_node_builder_new`].
/// * `node_storage` - Must be either a NULL pointer or a pointer to a valid [`iox2_node_storage_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `node_type` - The [`iox2_node_type_e`] for the node to be created.
/// * `node_handle_ptr` - A dangling [`iox2_node_h`] handle which will be initialized by this function call.
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
    assert!(!node_builder_handle.is_null());
    assert!(!node_handle_ptr.is_null());

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
    assert!(!node_handle.is_null());

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

// END iox2_node_builder_* C API

// BEGIN iox2_node_* C API

/// This function needs to be called to destroy the node!
///
/// # Arguments
///
/// * `node_handle` - A valid [`iox2_node_h`]
///
/// # Safety
///
/// The `node_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// The corresponding [`iox2_node_storage_t`] can be re-used with a call to [`iox2_node_builder_create`]!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_drop(node_handle: iox2_node_h) {
    assert!(!node_handle.is_null());

    unsafe {
        match (*node_handle).node_type {
            iox2_node_type_e::ZERO_COPY => {
                std::ptr::drop_in_place(
                    (*node_handle).node_assume_init::<zero_copy::Service>() as *mut _
                );
                ((*node_handle).deleter)(node_handle);
            }
            iox2_node_type_e::PROCESS_LOCAL => {
                std::ptr::drop_in_place(
                    (*node_handle).node_assume_init::<process_local::Service>() as *mut _,
                );
                ((*node_handle).deleter)(node_handle);
            }
        }
    }
}

// END iox2_node_* C API

#[cfg(test)]
mod test {
    use super::*;
    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn assert_storage_sizes() {
        // all const functions; if it compiles, the storage size is sufficient
        const _NODE_BUILDER_STORAGE_LAYOUT_CHECK: () =
            iox2_node_builder_storage_t::assert_storage_layout();
        const _NODE_STORAGE_LAYOUT_CHECK: () = iox2_node_storage_t::assert_storage_layout();
    }

    #[test]
    fn basic_node_api_test() {
        unsafe {
            let node_builder_handle = iox2_node_builder_new(std::ptr::null_mut());
            let mut node_handle: iox2_node_h = std::ptr::null_mut();
            let ret_val = iox2_node_builder_create(
                node_builder_handle,
                std::ptr::null_mut(),
                iox2_node_type_e::ZERO_COPY,
                &mut node_handle as *mut iox2_node_h,
            );

            assert_that!(ret_val, eq(IOX2_OK));
            assert_that!(node_handle, ne(std::ptr::null_mut()));

            iox2_node_drop(node_handle);
        }
    }
}
