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
    iox2_callback_progression_e, iox2_config_ptr, iox2_node_name_ptr, iox2_service_builder_h,
    iox2_service_builder_t, iox2_service_name_ptr, iox2_service_type_e, HandleToType, IntoCInt,
    ServiceBuilderUnion, IOX2_OK,
};
use crate::iox2_callback_context;

use iceoryx2::node::{NodeId, NodeListFailure, NodeView};
use iceoryx2::prelude::*;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::ffi::c_int;
use core::mem::ManuallyDrop;
use std::time::Duration;

// BEGIN type definition

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
#[derive(Copy, Clone)]
pub enum iox2_node_event_e {
    TICK = 0,
    TERMINATION_REQUEST,
    INTERRUPT_SIGNAL,
}

impl IntoCInt for NodeEvent {
    fn into_c_int(self) -> c_int {
        (match self {
            NodeEvent::Tick => iox2_node_event_e::TICK,
            NodeEvent::TerminationRequest => iox2_node_event_e::TERMINATION_REQUEST,
            NodeEvent::InterruptSignal => iox2_node_event_e::INTERRUPT_SIGNAL,
        }) as c_int
    }
}

pub(super) union NodeUnion {
    ipc: ManuallyDrop<Node<ipc::Service>>,
    local: ManuallyDrop<Node<local::Service>>,
}

impl NodeUnion {
    pub(super) fn new_ipc(node: Node<ipc::Service>) -> Self {
        Self {
            ipc: ManuallyDrop::new(node),
        }
    }
    pub(super) fn new_local(node: Node<local::Service>) -> Self {
        Self {
            local: ManuallyDrop::new(node),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<NodeUnion>
pub struct iox2_node_storage_t {
    internal: [u8; 16], // magic number obtained with size_of::<Option<NodeUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(NodeUnion)]
pub struct iox2_node_t {
    pub(super) service_type: iox2_service_type_e,
    pub(super) value: iox2_node_storage_t,
    pub(super) deleter: fn(*mut iox2_node_t),
}

impl iox2_node_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: NodeUnion,
        deleter: fn(*mut iox2_node_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_name_h_t;
/// The owning handle for `iox2_node_t`. Passing the handle to an function transfers the ownership.
pub type iox2_node_h = *mut iox2_name_h_t;

pub struct iox2_node_ref_h_t;
/// The non-owning handle for `iox2_node_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_node_ref_h = *mut iox2_node_ref_h_t;

impl HandleToType for iox2_node_h {
    type Target = *mut iox2_node_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_node_ref_h {
    type Target = *mut iox2_node_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
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

// NOTE check the README.md for using opaque types with renaming
/// The immutable pointer to the underlying `NodeId`
pub type iox2_node_id_ptr = *const NodeId;
/// The mutable pointer to the underlying `NodeId`
pub type iox2_node_id_mut_ptr = *mut NodeId;

/// The callback for [`iox2_node_list`]
///
/// # Arguments
///
/// * [`iox2_node_state_e`]
/// * [`iox2_node_id_ptr`]
/// * [`iox2_node_name_ptr`](crate::iox2_node_name_ptr) -> `NULL` for `iox2_node_state_e::INACCESSIBLE` and `iox2_node_state_e::UNDEFINED`
/// * [`iox2_config_ptr`](crate::iox2_config_ptr) -> `NULL` for `iox2_node_state_e::INACCESSIBLE` and `iox2_node_state_e::UNDEFINED`
/// * [`iox2_callback_context`] -> provided by the user to [`iox2_node_list`] and can be `NULL`
///
/// Returns a [`iox2_callback_progression_e`](crate::iox2_callback_progression_e)
pub type iox2_node_list_callback = extern "C" fn(
    iox2_node_state_e,
    iox2_node_id_ptr,
    iox2_node_name_ptr,
    iox2_config_ptr,
    iox2_callback_context,
) -> iox2_callback_progression_e;

// END type definition

// BEGIN C API

/// This function casts an owning [`iox2_node_h`] into a non-owning [`iox2_node_ref_h`]
///
/// # Arguments
///
/// * `node_handle` obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)
///
/// Returns a [`iox2_node_ref_h`]
///
/// # Safety
///
/// * The `node_handle` must be a valid handle.
/// * The `node_handle` is still valid after the call to this function.
#[no_mangle]
pub unsafe extern "C" fn iox2_cast_node_ref_h(node_handle: iox2_node_h) -> iox2_node_ref_h {
    debug_assert!(!node_handle.is_null());

    (*node_handle.as_type()).as_ref_handle()
}

/// Returns the [`iox2_node_name_ptr`](crate::iox2_node_name_ptr), an immutable pointer to the node name.
///
/// # Safety
///
/// * The `node_handle` must be valid and obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_name(node_handle: iox2_node_ref_h) -> iox2_node_name_ptr {
    debug_assert!(!node_handle.is_null());

    let node = &mut *node_handle.as_type();

    match node.service_type {
        iox2_service_type_e::IPC => node.value.as_ref().ipc.name(),
        iox2_service_type_e::LOCAL => node.value.as_ref().local.name(),
    }
}

/// Wait until the provided cycle time has passed and returns a [`iox2_node_event_e`] enum containing the event that
/// has occurred.
///
/// # Safety
///
/// * The `node_handle` must be valid and obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_wait(
    node_handle: iox2_node_ref_h,
    cycle_time_sec: u64,
    cycle_time_nsec: u32,
) -> c_int {
    debug_assert!(!node_handle.is_null());

    let node = &mut *node_handle.as_type();
    let cycle_time =
        Duration::from_secs(cycle_time_sec) + Duration::from_nanos(cycle_time_nsec as u64);
    match node.service_type {
        iox2_service_type_e::IPC => node.value.as_ref().ipc.wait(cycle_time),
        iox2_service_type_e::LOCAL => node.value.as_ref().local.wait(cycle_time),
    }
    .into_c_int()
}

/// Returns the [`iox2_config_ptr`](crate::iox2_config_ptr), an immutable pointer to the config.
///
/// # Safety
///
/// * The `node_handle` must be valid and obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_config(node_handle: iox2_node_ref_h) -> iox2_config_ptr {
    debug_assert!(!node_handle.is_null());

    let node = &mut *node_handle.as_type();

    match node.service_type {
        iox2_service_type_e::IPC => node.value.as_ref().ipc.config(),
        iox2_service_type_e::LOCAL => node.value.as_ref().local.config(),
    }
}

/// Returns the [`iox2_node_id_ptr`](crate::iox2_node_id_ptr), an immutable pointer to the node id.
///
/// # Safety
///
/// * The `node_handle` must be valid and obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_id(node_handle: iox2_node_ref_h) -> iox2_node_id_ptr {
    debug_assert!(!node_handle.is_null());
    todo!() // TODO: [#210] implement
}

fn iox2_node_list_impl<S: Service>(
    node_state: &NodeState<S>,
    callback: iox2_node_list_callback,
    callback_ctx: iox2_callback_context,
) -> CallbackProgression {
    match node_state {
        NodeState::Alive(alive_node_view) => {
            let (node_name, config) = alive_node_view
                .details()
                .as_ref()
                .map(|view| (view.name() as _, view.config() as _))
                .unwrap_or((std::ptr::null(), std::ptr::null()));
            callback(
                iox2_node_state_e::ALIVE,
                alive_node_view.id(),
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
                .map(|view| (view.name() as _, view.config() as _))
                .unwrap_or((std::ptr::null(), std::ptr::null()));
            callback(
                iox2_node_state_e::DEAD,
                dead_node_view.id(),
                node_name,
                config,
                callback_ctx,
            )
            .into()
        }
        NodeState::Inaccessible(ref node_id) => callback(
            iox2_node_state_e::INACCESSIBLE,
            node_id,
            std::ptr::null(),
            std::ptr::null(),
            callback_ctx,
        )
        .into(),
        NodeState::Undefined(ref node_id) => callback(
            iox2_node_state_e::UNDEFINED,
            node_id,
            std::ptr::null(),
            std::ptr::null(),
            callback_ctx,
        )
        .into(),
    }
}

/// Calls the callback repeatedly with an [`iox2_node_state_e`], [`iox2_node_id_ptr`], [´iox2_node_name_ptr´] and [`iox2_config_ptr`] for
/// all [`Node`](iceoryx2::node::Node)s in the system under a given [`Config`](iceoryx2::config::Config).
///
/// # Arguments
///
/// * `service_type` - A [`iox2_service_type_e`]
/// * `config_ptr` - A valid [`iox2_config_ptr`](crate::iox2_config_ptr)
/// * `callback` - A valid callback with [`iox2_node_list_callback`} signature
/// * `callback_ctx` - An optional callback context [`iox2_callback_context`} to e.g. store information across callback iterations
///
/// Returns IOX2_OK on success, an [`iox2_node_list_failure_e`] otherwise.
///
/// # Safety
///
/// * The `config_ptr` must be valid and obtained by ether [`iox2_node_config`] or [`iox2_config_global_config`](crate::iox2_config_global_config)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_list(
    service_type: iox2_service_type_e,
    config_ptr: iox2_config_ptr,
    callback: iox2_node_list_callback,
    callback_ctx: iox2_callback_context,
) -> c_int {
    debug_assert!(!config_ptr.is_null());

    let config = &*config_ptr;

    let list_result = match service_type {
        iox2_service_type_e::IPC => Node::<ipc::Service>::list(config, |node_state| {
            iox2_node_list_impl(&node_state, callback, callback_ctx)
        }),
        iox2_service_type_e::LOCAL => Node::<local::Service>::list(config, |node_state| {
            iox2_node_list_impl(&node_state, callback, callback_ctx)
        }),
    };

    match list_result {
        Ok(_) => IOX2_OK,
        Err(e) => e.into_c_int(),
    }
}

/// Instantiates a [`iox2_service_builder_h`] for a service with the provided name.
///
/// # Arguments
///
/// * `node_handle` - Must be a valid [`iox2_node_ref_h`] obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)
///   and casted by [`iox2_cast_node_ref_h`]
/// * `service_builder_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_service_builder_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
/// * `service_name_ptr` - Must be a valid [`iox2_service_name_ptr`] obtained by [`iox2_service_name_new`](crate::iox2_service_name_new)
///   and casted by [`iox2_cast_service_name_ptr`](crate::iox2_cast_service_name_ptr)
///
/// Returns the `iox2_service_builder_h` handle for the service builder.
///
/// # Safety
///
/// * The `node_handle` is still valid after the return of this function and can be use in another function call.
#[no_mangle]
pub unsafe extern "C" fn iox2_node_service_builder(
    node_handle: iox2_node_ref_h,
    service_builder_struct_ptr: *mut iox2_service_builder_t,
    service_name_ptr: iox2_service_name_ptr,
) -> iox2_service_builder_h {
    debug_assert!(!node_handle.is_null());
    debug_assert!(!service_name_ptr.is_null());

    let mut service_builder_struct_ptr = service_builder_struct_ptr;
    fn no_op(_: *mut iox2_service_builder_t) {}
    let mut deleter: fn(*mut iox2_service_builder_t) = no_op;
    if service_builder_struct_ptr.is_null() {
        service_builder_struct_ptr = iox2_service_builder_t::alloc();
        deleter = iox2_service_builder_t::dealloc;
    }
    debug_assert!(!service_builder_struct_ptr.is_null());

    let node = &mut *node_handle.as_type();
    match node.service_type {
        iox2_service_type_e::IPC => {
            let service_builder = node.value.as_ref().ipc.service_builder(&*service_name_ptr);
            (*service_builder_struct_ptr).init(
                node.service_type,
                ServiceBuilderUnion::new_ipc_base(service_builder),
                deleter,
            );
        }
        iox2_service_type_e::LOCAL => {
            let service_builder = node
                .value
                .as_ref()
                .local
                .service_builder(&*service_name_ptr);
            (*service_builder_struct_ptr).init(
                node.service_type,
                ServiceBuilderUnion::new_local_base(service_builder),
                deleter,
            );
        }
    };

    (*service_builder_struct_ptr).as_handle()
}

/// This function needs to be called to destroy the node!
///
/// # Arguments
///
/// * `node_handle` - A valid [`iox2_node_h`]
///
/// # Safety
///
/// * The `node_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_node_t`] can be re-used with a call to [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_drop(node_handle: iox2_node_h) {
    debug_assert!(!node_handle.is_null());

    let node = &mut *node_handle.as_type();

    match node.service_type {
        iox2_service_type_e::IPC => {
            ManuallyDrop::drop(&mut node.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            ManuallyDrop::drop(&mut node.value.as_mut().local);
        }
    }
    (node.deleter)(node);
}

// END C API
