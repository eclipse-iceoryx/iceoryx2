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
    iox2_node_name_t, iox2_service_builder_h, iox2_service_builder_storage_t, iox2_service_name_h,
};

use iceoryx2::prelude::*;
use iceoryx2::service;
use iceoryx2_bb_elementary::math::max;
use iceoryx2_bb_elementary::static_assert::*;

use core::ffi::c_int;
use core::mem::{align_of, size_of, MaybeUninit};
use std::alloc::{alloc, dealloc, Layout};

// BEGIN type definition

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
    pub(crate) node_type: iox2_node_type_e,
    pub(crate) internal: iox2_node_storage_internal_t,
    pub(crate) deleter: fn(*mut iox2_node_storage_t),
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
        static_assert_ge::<{ align_of::<iox2_node_storage_internal_t>() }, { MAX_NODE_ALIGNMENT }>(
        );
        static_assert_ge::<{ size_of::<iox2_node_storage_internal_t>() }, { MAX_NODE_SIZE }>();
    }

    pub(crate) fn node_maybe_uninit<Service: service::Service>(
        &mut self,
    ) -> &mut MaybeUninit<Node<Service>> {
        iox2_node_storage_t::assert_storage_layout();

        unsafe {
            &mut *(&mut self.internal as *mut iox2_node_storage_internal_t)
                .cast::<MaybeUninit<Node<Service>>>()
        }
    }

    pub(crate) unsafe fn node_assume_init<Service: service::Service>(
        &mut self,
    ) -> &mut Node<Service> {
        self.node_maybe_uninit().assume_init_mut()
    }

    pub(crate) fn alloc() -> *mut iox2_node_storage_t {
        unsafe { alloc(Layout::new::<iox2_node_storage_t>()) as *mut iox2_node_storage_t }
    }
    pub(crate) fn dealloc(storage: *mut iox2_node_storage_t) {
        unsafe {
            dealloc(storage as *mut _, Layout::new::<iox2_node_storage_t>());
        }
    }
}

// END type definition

// BEGIN C API

/// Returns the [`iox2_node_name_t`](crate::iox2_node_name_t), an immutable handle to the node name.
///
/// # Safety
///
/// The `node_handle` must be valid and obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_name(node_handle: iox2_node_h) -> iox2_node_name_t {
    debug_assert!(!node_handle.is_null());
    todo!() // TODO: [#210] implement
}

pub type iox2_config_t = *const (); // TODO: [#210] implement in config.rs
/// Returns the immutable [`iox2_config_t`] handle that the [`iox2_node_h`] will use to create any iceoryx2 entity.
///
/// # Safety
///
/// The `node_handle` must be valid and obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_config(node_handle: iox2_node_h) -> iox2_config_t {
    debug_assert!(!node_handle.is_null());
    todo!() // TODO: [#210] implement
}

pub type iox2_unique_system_id_t = *const (); // TODO: [#210] implement in unique_system_id.rs
/// Returns the immutable [`iox2_unique_system_id_t`] handle of the [`iox2_node_h`].
///
/// # Safety
///
/// The `node_handle` must be valid and obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_id(node_handle: iox2_node_h) -> iox2_unique_system_id_t {
    debug_assert!(!node_handle.is_null());
    todo!() // TODO: [#210] implement
}

pub type iox2_node_state_t = *mut (); // TODO: [#210] implement in node_state.rs
pub type iox2_node_list_failure_e = c_int; // TODO: [#210] implement in this file
/// Call the callback repeatedly with an immutable [`iox2_node_state_t`] handle for all [`Node`](iceoryx2::node::Node)s
/// in the system under a given [`Config`](iceoryx2::config::Config).
///
/// # Arguments
///
/// * `node_handle` - A valid [`iox2_node_h`]
/// * `config_handle` - A valid [`iox2_config_t`]
/// * `callback` - A valid callback with ??? signature
///
/// Returns IOX2_OK on success, an [`iox2_node_list_failure_e`] otherwise.
///
/// # Safety
///
/// The `node_handle` must be valid and obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_nodel_list(
    node_handle: iox2_node_h,
    config_handle: iox2_config_t,
) -> c_int {
    debug_assert!(!node_handle.is_null());
    debug_assert!(!config_handle.is_null());
    todo!() // TODO: [#210] implement

    // IOX2_OK
}

#[no_mangle]
pub extern "C" fn iox2_service_name_new() {}
/// Instantiates a [`iox2_service_builder_h`] for a service with the provided name.
///
/// # Safety
///
/// The `node_handle` must be valid and obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
/// The `service_name_handle` must be valid and obtained by [`iox2_service_name_new`]!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_service_builder(
    node_handle: iox2_node_h,
    _service_builder_storage: *mut iox2_service_builder_storage_t,
    service_name_handle: iox2_service_name_h,
) -> iox2_service_builder_h {
    debug_assert!(!node_handle.is_null());
    debug_assert!(!service_name_handle.is_null());
    todo!() // TODO: [#210] implement
}

/// This function needs to be called to destroy the node!
///
/// # Arguments
///
/// * `node_handle` - A valid [`iox2_node_h`]
///
/// # Safety
///
/// The `node_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// The corresponding [`iox2_node_storage_t`] can be re-used with a call to [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_drop(node_handle: iox2_node_h) {
    debug_assert!(!node_handle.is_null());

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

// END C API

#[cfg(test)]
mod test {
    use crate::*;
    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn assert_storage_sizes() {
        // all const functions; if it compiles, the storage size is sufficient
        const _STORAGE_LAYOUT_CHECK: () = iox2_node_storage_t::assert_storage_layout();
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
