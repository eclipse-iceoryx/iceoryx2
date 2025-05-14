// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use iceoryx2::service::dynamic_config::event::ListenerDetails;

use super::{iox2_node_id_ptr, iox2_unique_listener_id_h, iox2_unique_listener_id_t};

/// The immutable pointer to the underlying `ListenerDetails`
pub type iox2_listener_details_ptr = *const ListenerDetails;

/// Returns the unique port id of the listener.
///
/// # Safety
///
/// * `handle` valid pointer to the listener details
/// * `id_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_unique_listener_id_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
/// * `id_handle_ptr` valid pointer to a [`iox2_unique_listener_id_h`].
#[no_mangle]
pub unsafe extern "C" fn iox2_listener_details_listener_id(
    handle: iox2_listener_details_ptr,
    id_struct_ptr: *mut iox2_unique_listener_id_t,
    id_handle_ptr: *mut iox2_unique_listener_id_h,
) {
    debug_assert!(!handle.is_null());
    debug_assert!(!id_handle_ptr.is_null());

    fn no_op(_: *mut iox2_unique_listener_id_t) {}
    let mut deleter: fn(*mut iox2_unique_listener_id_t) = no_op;
    let mut storage_ptr = id_struct_ptr;
    if id_struct_ptr.is_null() {
        deleter = iox2_unique_listener_id_t::dealloc;
        storage_ptr = iox2_unique_listener_id_t::alloc();
    }
    debug_assert!(!storage_ptr.is_null());

    let id = (*handle).listener_id;

    (*storage_ptr).init(id, deleter);
    *id_handle_ptr = (*storage_ptr).as_handle();
}

/// Returns the [`iox2_node_id_ptr`](crate::iox2_node_id_ptr), an immutable pointer to the node id.
///
/// # Safety
///
/// * `handle` valid pointer to the listener details
#[no_mangle]
pub unsafe extern "C" fn iox2_listener_details_node_id(
    handle: iox2_listener_details_ptr,
) -> iox2_node_id_ptr {
    debug_assert!(!handle.is_null());

    &(*handle).node_id
}
