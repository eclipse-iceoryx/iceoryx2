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

// use super::{IntoCInt, IOX2_OK};

use iceoryx2::prelude::*;
use iceoryx2::service;
use iceoryx2_bb_elementary::math::max;
use iceoryx2_bb_elementary::static_assert::*;

// use core::ffi::c_int;
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
