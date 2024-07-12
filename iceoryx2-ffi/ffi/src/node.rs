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
    iox2_callback_progression_e, iox2_config_h, iox2_node_name_h, iox2_service_builder_mut_h,
    iox2_service_builder_storage_t, iox2_service_name_mut_h, iox2_service_type_e, IntoCInt,
    IOX2_OK,
};

use iceoryx2::node::{NodeListFailure, NodeView};
use iceoryx2::prelude::*;
use iceoryx2::service;
use iceoryx2_bb_elementary::math::max;
use iceoryx2_bb_elementary::static_assert::*;

use core::ffi::{c_int, c_void};
use core::mem::{align_of, size_of, MaybeUninit};
use std::alloc::{alloc, dealloc, Layout};

// BEGIN type definition

#[repr(C)]
#[repr(align(8))] // magic number; the larger one of align_of::<Node<zero_copy::Service>>() and align_of::<Node<process_local::Service>>()
pub struct iox2_node_storage_internal_t {
    internal: [u8; 8], // magic number; the larger one of size_of::<Node<zero_copy::Service>>() and size_of::<Node<process_local::Service>>()
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_node_list_failure_e {
    INSUFFICIENT_PERMISSIONS = IOX2_OK as isize + 1,
    INTERRUPT,
    INTERNAL_ERROR,
}

impl IntoCInt for NodeListFailure {
    fn into_c_int(self) -> c_int {
        (match self {
            NodeListFailure::InsufficientPermissions => {
                iox2_node_list_failure_e::INSUFFICIENT_PERMISSIONS
            }
            NodeListFailure::Interrupt => iox2_node_list_failure_e::INTERRUPT,
            NodeListFailure::InternalError => iox2_node_list_failure_e::INTERNAL_ERROR,
        }) as c_int
    }
}

#[repr(C)]
pub struct iox2_node_storage_t {
    pub(crate) service_type: iox2_service_type_e,
    pub(crate) internal: iox2_node_storage_internal_t,
    pub(crate) deleter: fn(*mut iox2_node_storage_t),
}

/// The handle to use for the `iox2_node_*` functions which mutate the node
pub type iox2_node_mut_h = *mut iox2_node_storage_t;

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

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_node_state_e {
    ALIVE,
    DEAD,
    INACCESSIBLE,
    UNDEFINED,
}

// TODO: [#210] implement
pub type iox2_node_id_h = *const c_void;

/// An alias to a `void *` which can be used to pass arbitrary data to the callback
pub type iox2_node_list_callback_context = *mut c_void;

/// The callback for [`iox2_node_list`]
///
/// # Arguments
///
/// * [`iox2_node_state_e`]
/// * [`iox2_node_id_h`]
/// * [`iox2_node_name_h`](crate::iox2_node_name_h) -> `NULL` for `iox2_node_state_e::INACCESSIBLE` and `iox2_node_state_e::UNDEFINED`
/// * [`iox2_config_h`](crate::iox2_config_h) -> `NULL` for `iox2_node_state_e::INACCESSIBLE` and `iox2_node_state_e::UNDEFINED`
/// * [`iox2_node_list_callback_context`] -> provided by the user to [`iox2_node_list`] and can be `NULL`
///
/// Returns a [`iox2_callback_progression_e`](crate::iox2_callback_progression_e)
pub type iox2_node_list_callback = extern "C" fn(
    iox2_node_state_e,
    iox2_node_id_h,
    iox2_node_name_h,
    iox2_config_h,
    iox2_node_list_callback_context,
) -> iox2_callback_progression_e;

// END type definition

// BEGIN C API

/// Returns the [`iox2_node_name_h`](crate::iox2_node_name_h), an immutable handle to the node name.
///
/// # Safety
///
/// The `node_handle` must be valid and obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_name(node_handle: iox2_node_mut_h) -> iox2_node_name_h {
    debug_assert!(!node_handle.is_null());

    unsafe {
        match (*node_handle).service_type {
            iox2_service_type_e::IPC => (*node_handle)
                .node_assume_init::<zero_copy::Service>()
                .name() as *const _ as *const _,
            iox2_service_type_e::LOCAL => (*node_handle)
                .node_assume_init::<process_local::Service>()
                .name() as *const _ as *const _,
        }
    }
}

/// Returns the immutable [`iox2_config_h`](crate::iox2_config_h) handle that the [`iox2_node_mut_h`] will use to create any iceoryx2 entity.
///
/// # Safety
///
/// The `node_handle` must be valid and obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_config(node_handle: iox2_node_mut_h) -> iox2_config_h {
    debug_assert!(!node_handle.is_null());

    unsafe {
        match (*node_handle).service_type {
            iox2_service_type_e::IPC => (*node_handle)
                .node_assume_init::<zero_copy::Service>()
                .config() as *const _ as *const _,
            iox2_service_type_e::LOCAL => (*node_handle)
                .node_assume_init::<process_local::Service>()
                .config() as *const _ as *const _,
        }
    }
}

/// Returns the immutable [`iox2_node_id_h`] handle of the [`iox2_node_mut_h`].
///
/// # Safety
///
/// The `node_handle` must be valid and obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_id(node_handle: iox2_node_mut_h) -> iox2_node_id_h {
    debug_assert!(!node_handle.is_null());
    todo!() // TODO: [#210] implement
}

fn iox2_node_list_impl<S: Service>(
    node_state: &NodeState<S>,
    callback: iox2_node_list_callback,
    callback_ctx: iox2_node_list_callback_context,
) -> CallbackProgression {
    match node_state {
        NodeState::Alive(alive_node_view) => {
            let (node_name, config) = alive_node_view
                .details()
                .as_ref()
                .map(|view| {
                    (
                        view.name() as *const _ as *const _,
                        view.config() as *const _ as *const _,
                    )
                })
                .unwrap_or((std::ptr::null(), std::ptr::null()));
            callback(
                iox2_node_state_e::ALIVE,
                alive_node_view.id() as *const _ as *const _,
                node_name,
                config,
                callback_ctx,
            )
            .into()
        }
        NodeState::Dead(dead_node_view) => {
            let (node_name, config) = dead_node_view
                .details()
                .as_ref()
                .map(|view| {
                    (
                        view.name() as *const _ as *const _,
                        view.config() as *const _ as *const _,
                    )
                })
                .unwrap_or((std::ptr::null(), std::ptr::null()));
            callback(
                iox2_node_state_e::DEAD,
                dead_node_view.id() as *const _ as *const _,
                node_name,
                config,
                callback_ctx,
            )
            .into()
        }
        NodeState::Inaccessible(ref node_id) => callback(
            iox2_node_state_e::INACCESSIBLE,
            node_id as *const _ as *const _,
            std::ptr::null(),
            std::ptr::null(),
            callback_ctx,
        )
        .into(),
        NodeState::Undefined(ref node_id) => callback(
            iox2_node_state_e::UNDEFINED,
            node_id as *const _ as *const _,
            std::ptr::null(),
            std::ptr::null(),
            callback_ctx,
        )
        .into(),
    }
}

/// Calls the callback repeatedly with an [`iox2_node_state_e`], [`iox2_node_id_h`], [´iox2_node_name_h´] and [`iox2_config_h`] for
/// all [`Node`](iceoryx2::node::Node)s in the system under a given [`Config`](iceoryx2::config::Config).
///
/// # Arguments
///
/// * `service_type` - A [`iox2_service_type_e`]
/// * `config_handle` - A valid [`iox2_config_h`](crate::iox2_config_h)
/// * `callback` - A valid callback with [`iox2_node_list_callback`} signature
/// * `callback_ctx` - An optional callback context [`iox2_node_list_callback_context`} to e.g. store information across callback iterations
///
/// Returns IOX2_OK on success, an [`iox2_node_list_failure_e`] otherwise.
///
/// # Safety
///
/// The `config_handle` must be valid and obtained by ether [`iox2_node_config`] or [`iox2_config_global_config`](crate::iox2_config_global_config)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_list(
    service_type: iox2_service_type_e,
    config_handle: iox2_config_h,
    callback: iox2_node_list_callback,
    callback_ctx: iox2_node_list_callback_context,
) -> c_int {
    debug_assert!(!config_handle.is_null());

    let config = &*(config_handle as *const _);

    let list_result = match service_type {
        iox2_service_type_e::IPC => Node::<zero_copy::Service>::list(config, |node_state| {
            iox2_node_list_impl(&node_state, callback, callback_ctx)
        }),
        iox2_service_type_e::LOCAL => Node::<process_local::Service>::list(config, |node_state| {
            iox2_node_list_impl(&node_state, callback, callback_ctx)
        }),
    };

    match list_result {
        Ok(_) => IOX2_OK,
        Err(e) => e.into_c_int(),
    }
}

#[no_mangle]
pub extern "C" fn iox2_service_name_new() {}
/// Instantiates a [`iox2_service_builder_mut_h`] for a service with the provided name.
///
/// # Safety
///
/// The `node_handle` must be valid and obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
/// The `service_name_handle` must be valid and obtained by [`iox2_service_name_new`]!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_service_builder(
    node_handle: iox2_node_mut_h,
    _service_builder_storage: *mut iox2_service_builder_storage_t,
    service_name_handle: iox2_service_name_mut_h,
) -> iox2_service_builder_mut_h {
    debug_assert!(!node_handle.is_null());
    debug_assert!(!service_name_handle.is_null());
    todo!() // TODO: [#210] implement
}

/// This function needs to be called to destroy the node!
///
/// # Arguments
///
/// * `node_handle` - A valid [`iox2_node_mut_h`]
///
/// # Safety
///
/// The `node_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// The corresponding [`iox2_node_storage_t`] can be re-used with a call to [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_drop(node_handle: iox2_node_mut_h) {
    debug_assert!(!node_handle.is_null());

    unsafe {
        match (*node_handle).service_type {
            iox2_service_type_e::IPC => {
                std::ptr::drop_in_place(
                    (*node_handle).node_assume_init::<zero_copy::Service>() as *mut _
                );
                ((*node_handle).deleter)(node_handle);
            }
            iox2_service_type_e::LOCAL => {
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

    fn create_sut_node() -> iox2_node_mut_h {
        unsafe {
            let node_builder_handle = iox2_node_builder_new(std::ptr::null_mut());

            let mut node_name_handle_mut = std::ptr::null_mut();
            let node_name = "hypnotoad";
            let ret_val = iox2_node_name_new(
                std::ptr::null_mut(),
                node_name.as_ptr() as *const _,
                node_name.len() as _,
                &mut node_name_handle_mut,
            );
            assert_that!(ret_val, eq(IOX2_OK));
            iox2_node_builder_set_name(node_builder_handle, node_name_handle_mut);

            let mut node_handle: iox2_node_mut_h = std::ptr::null_mut();
            let ret_val = iox2_node_builder_create(
                node_builder_handle,
                std::ptr::null_mut(),
                iox2_service_type_e::IPC,
                &mut node_handle as *mut iox2_node_mut_h,
            );

            assert_that!(ret_val, eq(IOX2_OK));

            node_handle
        }
    }

    #[test]
    fn basic_node_api_test() {
        unsafe {
            let node_handle = create_sut_node();

            assert_that!(node_handle, ne(std::ptr::null_mut()));

            iox2_node_drop(node_handle);
        }
    }

    #[test]
    fn basic_node_config_test() {
        unsafe {
            let node_handle = create_sut_node();
            let expected_config = (*node_handle)
                .node_assume_init::<zero_copy::Service>()
                .config();

            let config = iox2_node_config(node_handle);

            assert_that!(*(config as *const Config), eq(*expected_config));

            iox2_node_drop(node_handle);
        }
    }

    #[test]
    fn basic_node_name_test() {
        unsafe {
            let node_handle = create_sut_node();
            let expected_node_name = (*node_handle)
                .node_assume_init::<zero_copy::Service>()
                .name();
            assert_that!(expected_node_name.as_str(), eq("hypnotoad"));

            let node_name = iox2_node_name(node_handle);

            assert_that!(*(node_name as *const NodeName), eq(*expected_node_name));

            iox2_node_drop(node_handle);
        }
    }

    #[derive(Default)]
    struct NodeListCtx {
        alive: u64,
        dead: u64,
        inaccessible: u64,
        undefined: u64,
    }

    extern "C" fn node_list_callback(
        node_state: iox2_node_state_e,
        _node_id: iox2_node_id_h,
        _node_name: iox2_node_name_h,
        _config: iox2_config_h,
        ctx: iox2_node_list_callback_context,
    ) -> iox2_callback_progression_e {
        let ctx = unsafe { &mut *(ctx as *mut NodeListCtx) };

        match node_state {
            iox2_node_state_e::ALIVE => {
                ctx.alive += 1;
            }
            iox2_node_state_e::DEAD => {
                ctx.dead += 1;
            }
            iox2_node_state_e::INACCESSIBLE => {
                ctx.inaccessible += 1;
            }
            iox2_node_state_e::UNDEFINED => {
                ctx.undefined += 1;
            }
        }

        iox2_callback_progression_e::CONTINUE
    }

    #[test]
    fn basic_node_list_test() {
        unsafe {
            let mut ctx = NodeListCtx::default();
            let node_handle = create_sut_node();
            let config = iox2_node_config(node_handle);

            let ret_val = iox2_node_list(
                iox2_service_type_e::IPC,
                config,
                node_list_callback,
                &mut ctx as *mut _ as *mut _,
            );

            iox2_node_drop(node_handle);

            assert_that!(ret_val, eq(IOX2_OK));

            assert_that!(ctx.alive, eq(1));
            assert_that!(ctx.dead, eq(0));
            assert_that!(ctx.inaccessible, eq(0));
            assert_that!(ctx.undefined, eq(0));
        }
    }
}
