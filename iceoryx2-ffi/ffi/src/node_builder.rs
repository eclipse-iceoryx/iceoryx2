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
    iox2_node_h, iox2_node_name_drop, iox2_node_name_h, iox2_node_name_t, iox2_node_t,
    iox2_service_type_e, IntoCInt, IOX2_OK,
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
pub struct iox2_node_builder_storage_t {
    internal: [u8; 18432], // magic number obtained with size_of::<NodeBuilder>()
}

impl iox2_node_builder_storage_t {
    const fn assert_storage_layout() {
        static_assert_ge::<
            { align_of::<iox2_node_builder_storage_t>() },
            { align_of::<Option<NodeBuilder>>() },
        >();
        static_assert_ge::<
            { size_of::<iox2_node_builder_storage_t>() },
            { size_of::<Option<NodeBuilder>>() },
        >();
    }

    fn init(&mut self, node_builder: NodeBuilder) {
        iox2_node_builder_storage_t::assert_storage_layout();

        unsafe { &mut *(self as *mut Self).cast::<MaybeUninit<Option<NodeBuilder>>>() }
            .write(Some(node_builder));
    }

    unsafe fn as_option_mut(&mut self) -> &mut Option<NodeBuilder> {
        &mut *(self as *mut Self).cast::<Option<NodeBuilder>>()
    }

    unsafe fn _os_option_ref(&self) -> &Option<NodeBuilder> {
        &*(self as *const Self).cast::<Option<NodeBuilder>>()
    }

    unsafe fn _as_mut(&mut self) -> &mut NodeBuilder {
        self.as_option_mut().as_mut().unwrap()
    }

    unsafe fn _as_ref(&self) -> &NodeBuilder {
        self._os_option_ref().as_ref().unwrap()
    }
}

#[repr(C)]
pub struct iox2_node_builder_t {
    /// cbindgen:rename=internal
    node_builder: iox2_node_builder_storage_t,
    deleter: fn(*mut iox2_node_builder_t),
}

impl iox2_node_builder_t {
    pub(crate) fn cast(node_builder: iox2_node_builder_h) -> *mut Self {
        node_builder as *mut _ as *mut Self
    }
    pub(crate) fn cast_from_ref(node_builder: iox2_node_builder_ref_h) -> *mut Self {
        node_builder as *mut _ as *mut Self
    }

    pub(crate) fn take(&mut self) -> Option<NodeBuilder> {
        unsafe { self.node_builder.as_option_mut().take() }
    }

    pub(crate) fn set(&mut self, node_builder: NodeBuilder) {
        unsafe { *self.node_builder.as_option_mut() = Some(node_builder) }
    }

    fn alloc() -> *mut iox2_node_builder_t {
        unsafe { alloc(Layout::new::<iox2_node_builder_t>()) as *mut iox2_node_builder_t }
    }
    fn dealloc(storage: *mut iox2_node_builder_t) {
        unsafe {
            dealloc(storage as *mut _, Layout::new::<iox2_node_builder_t>());
        }
    }
}

pub struct iox2_node_builder_h_t;
/// The owning handle for `iox2_node_builder_t`. Passing the handle to an function transfers the ownership.
pub type iox2_node_builder_h = *mut iox2_node_builder_h_t;

pub struct iox2_node_builder_ref_h_t;
/// The non-owning handle for `iox2_node_builder_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_node_builder_ref_h = *mut iox2_node_builder_ref_h_t;

// END type definition

// BEGIN C API

/// Creates a builder for nodes
///
/// # Arguments
///
/// * `node_builder_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_node_builder_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
///
/// # Returns
///
/// A [`iox2_node_builder_h`] handle to build the actual node.
///
/// # Safety
///
/// The same [`iox2_node_builder_t`] cannot be used in subsequent calls to this function, unless [`iox2_node_builder_create`] was called before!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_builder_new(
    node_builder_struct_ptr: *mut iox2_node_builder_t,
) -> iox2_node_builder_h {
    let mut node_builder_struct_ptr = node_builder_struct_ptr;
    fn no_op(_: *mut iox2_node_builder_t) {}
    let mut deleter: fn(*mut iox2_node_builder_t) = no_op;
    if node_builder_struct_ptr.is_null() {
        node_builder_struct_ptr = iox2_node_builder_t::alloc();
        deleter = iox2_node_builder_t::dealloc;
    }
    debug_assert!(!node_builder_struct_ptr.is_null());

    (*node_builder_struct_ptr).deleter = deleter;
    (*node_builder_struct_ptr)
        .node_builder
        .init(NodeBuilder::new());

    node_builder_struct_ptr as *mut _ as *mut _
}

/// This function casts an owning [`iox2_node_builder_h`] into a non-owning [`iox2_node_builder_ref_h`]
///
/// # Arguments
///
/// * `node_builder_handle` obtained by [`iox2_node_builder_new`]
///
/// Returns a [`iox2_node_builder_ref_h`]
///
/// # Safety
///
/// The `node_builder_handle` must be a valid handle.
/// The `node_builder_handle` is still valid after the call to this function.
#[no_mangle]
pub unsafe extern "C" fn iox2_cast_node_builder_ref_h(
    node_builder_handle: iox2_node_builder_h,
) -> iox2_node_builder_ref_h {
    debug_assert!(!node_builder_handle.is_null());

    node_builder_handle as *mut _ as _
}

/// Sets the node name for the builder
///
/// # Arguments
///
/// * `node_builder_handle` - Must be a valid [`iox2_node_builder_ref_h`] obtained by [`iox2_node_builder_new`] and casted by [`iox2_cast_node_builder_ref_h`].
/// * `node_name_handle` - Must be a valid [`iox2_node_name_h`] obtained by [`iox2_node_name_new`](crate::iox2_node_name_new).
///
/// Returns IOX2_OK
///
/// # Safety
///
/// `node_builder_handle` as well as `node_name_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_node_builder_set_name(
    node_builder_handle: iox2_node_builder_ref_h,
    node_name_handle: iox2_node_name_h,
) -> c_int {
    debug_assert!(!node_builder_handle.is_null());
    debug_assert!(!node_name_handle.is_null());

    let node_name_struct = &mut *iox2_node_name_t::cast(node_name_handle);
    let node_name = node_name_struct.take().unwrap();
    iox2_node_name_drop(node_name_handle);

    let node_builder_struct = &mut *iox2_node_builder_t::cast_from_ref(node_builder_handle);

    let node_builder = node_builder_struct.take().unwrap();
    let node_builder = node_builder.name(node_name);
    node_builder_struct.set(node_builder);

    IOX2_OK
}

#[no_mangle]
pub extern "C" fn iox2_node_builder_set_config(
    node_builder_handle: iox2_node_builder_ref_h,
) -> c_int {
    debug_assert!(!node_builder_handle.is_null());
    todo!() // TODO: [#210] implement

    // IOX2_OK
}

// intentionally not public API
unsafe fn iox2_node_builder_drop(node_builder_handle: iox2_node_builder_h) {
    debug_assert!(!node_builder_handle.is_null());

    let node_builder_struct = &mut (*iox2_node_builder_t::cast(node_builder_handle));
    std::ptr::drop_in_place(node_builder_struct.node_builder.as_option_mut() as *mut _);
    (node_builder_struct.deleter)(node_builder_struct);
}

/// Creates a node and consumes the builder
///
/// # Arguments
///
/// * `node_builder_handle` - Must be a valid [`iox2_node_builder_h`] obtained by [`iox2_node_builder_new`].
/// * `node_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_node_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `service_type` - The [`iox2_service_type_e`] for the node to be created.
/// * `node_handle_ptr` - An uninitialized or dangling [`iox2_node_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_node_creation_failure_e`] otherwise.
///
/// # Safety
///
/// The `node_builder_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// The corresponding [`iox2_node_builder_t`] can be re-used with a call to [`iox2_node_builder_new`]!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_builder_create(
    node_builder_handle: iox2_node_builder_h,
    node_struct_ptr: *mut iox2_node_t,
    service_type: iox2_service_type_e,
    node_handle_ptr: *mut iox2_node_h,
) -> c_int {
    debug_assert!(!node_builder_handle.is_null());
    debug_assert!(!node_handle_ptr.is_null());

    let node_builder_struct = &mut *iox2_node_builder_t::cast(node_builder_handle);
    let node_builder = node_builder_struct.take().unwrap();
    iox2_node_builder_drop(node_builder_handle);

    let mut node_struct_ptr = node_struct_ptr;
    fn no_op(_: *mut iox2_node_t) {}
    let mut deleter: fn(*mut iox2_node_t) = no_op;
    if node_struct_ptr.is_null() {
        node_struct_ptr = iox2_node_t::alloc();
        deleter = iox2_node_t::dealloc;
    }
    debug_assert!(!node_struct_ptr.is_null());

    match service_type {
        iox2_service_type_e::IPC => match node_builder.create::<zero_copy::Service>() {
            Ok(node) => unsafe {
                (*node_struct_ptr).init(service_type, node, deleter);
            },
            Err(error) => {
                return error.into_c_int();
            }
        },
        iox2_service_type_e::LOCAL => match node_builder.create::<process_local::Service>() {
            Ok(node) => unsafe {
                (*node_struct_ptr).init(service_type, node, deleter);
            },
            Err(error) => {
                return error.into_c_int();
            }
        },
    }

    *node_handle_ptr = node_struct_ptr as *mut _ as *mut _;

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
                iox2_service_type_e::LOCAL,
                &mut node_handle as *mut iox2_node_h,
            );

            assert_that!(ret_val, eq(IOX2_OK));
            assert_that!(node_handle, ne(std::ptr::null_mut()));

            iox2_node_drop(node_handle);
        }
    }
}
