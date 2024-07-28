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
    iox2_node_h, iox2_node_name_ptr, iox2_node_t, iox2_service_type_e, HandleToType, IntoCInt,
    NodeUnion, IOX2_OK,
};

use iceoryx2::node::NodeCreationFailure;
use iceoryx2::prelude::*;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::ffi::c_int;

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
#[repr(align(8))] // alignment of Option<NodeBuilder>
pub struct iox2_node_builder_storage_t {
    internal: [u8; 18696], // magic number obtained with size_of::<NodeBuilder>()
}

#[repr(C)]
#[iceoryx2_ffi(NodeBuilder)]
pub struct iox2_node_builder_t {
    value: iox2_node_builder_storage_t,
    deleter: fn(*mut iox2_node_builder_t),
}

pub struct iox2_node_builder_h_t;
/// The owning handle for `iox2_node_builder_t`. Passing the handle to an function transfers the ownership.
pub type iox2_node_builder_h = *mut iox2_node_builder_h_t;

pub struct iox2_node_builder_ref_h_t;
/// The non-owning handle for `iox2_node_builder_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_node_builder_ref_h = *mut iox2_node_builder_ref_h_t;

impl HandleToType for iox2_node_builder_h {
    type Target = *mut iox2_node_builder_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_node_builder_ref_h {
    type Target = *mut iox2_node_builder_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

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
/// * The same [`iox2_node_builder_t`] cannot be used in subsequent calls to this function, unless [`iox2_node_builder_create`] was called before!
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
    (*node_builder_struct_ptr).value.init(NodeBuilder::new());

    (*node_builder_struct_ptr).as_handle()
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
/// * The `node_builder_handle` must be a valid handle.
/// * The `node_builder_handle` is still valid after the call to this function.
#[no_mangle]
pub unsafe extern "C" fn iox2_cast_node_builder_ref_h(
    node_builder_handle: iox2_node_builder_h,
) -> iox2_node_builder_ref_h {
    debug_assert!(!node_builder_handle.is_null());

    (*node_builder_handle.as_type()).as_ref_handle()
}

/// Sets the node name for the builder
///
/// # Arguments
///
/// * `node_builder_handle` - Must be a valid [`iox2_node_builder_ref_h`] obtained by [`iox2_node_builder_new`] and casted by [`iox2_cast_node_builder_ref_h`].
/// * `node_name_ptr` - Must be a valid [`iox2_node_name_ptr`], e.g. obtained by [`iox2_node_name_new`](crate::iox2_node_name_new) and converted
///    by [`iox2_cast_node_name_ptr`](crate::iox2_cast_node_name_ptr)
///
/// Returns IOX2_OK
///
/// # Safety
///
/// * `node_builder_handle` as well as `node_name_ptr` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_node_builder_set_name(
    node_builder_handle: iox2_node_builder_ref_h,
    node_name_ptr: iox2_node_name_ptr,
) -> c_int {
    debug_assert!(!node_builder_handle.is_null());
    debug_assert!(!node_name_ptr.is_null());

    let node_builder_struct = &mut *node_builder_handle.as_type();

    let node_builder = node_builder_struct.take().unwrap();
    let node_builder = node_builder.name(&*node_name_ptr);
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

    let node_builder = &mut *node_builder_handle.as_type();
    std::ptr::drop_in_place(node_builder.value.as_option_mut());
    (node_builder.deleter)(node_builder);
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
/// * The `node_builder_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_node_builder_t`] can be re-used with a call to [`iox2_node_builder_new`]!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_builder_create(
    node_builder_handle: iox2_node_builder_h,
    node_struct_ptr: *mut iox2_node_t,
    service_type: iox2_service_type_e,
    node_handle_ptr: *mut iox2_node_h,
) -> c_int {
    debug_assert!(!node_builder_handle.is_null());
    debug_assert!(!node_handle_ptr.is_null());

    match service_type as usize {
        0 => (),
        1 => (),
        _ => fatal_panic!(from "iox2_node_builder_create",
                            "The provided service_type has an invalid value."),
    }

    let node_builder_struct = &mut *node_builder_handle.as_type();
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
        iox2_service_type_e::IPC => match node_builder.create::<ipc::Service>() {
            Ok(node) => unsafe {
                (*node_struct_ptr).init(service_type, NodeUnion::new_ipc(node), deleter);
            },
            Err(error) => {
                return error.into_c_int();
            }
        },
        iox2_service_type_e::LOCAL => match node_builder.create::<local::Service>() {
            Ok(node) => unsafe {
                (*node_struct_ptr).init(service_type, NodeUnion::new_local(node), deleter);
            },
            Err(error) => {
                return error.into_c_int();
            }
        },
    }

    *node_handle_ptr = (*node_struct_ptr).as_handle();

    IOX2_OK
}

// END C API
