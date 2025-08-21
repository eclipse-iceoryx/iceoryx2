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
    iox2_callback_context, iox2_callback_progression_e, iox2_config_ptr, iox2_node_name_ptr,
    iox2_service_builder_h, iox2_service_builder_t, iox2_service_name_ptr, iox2_service_type_e,
    AssertNonNullHandle, HandleToType, IntoCInt, ServiceBuilderUnion, IOX2_OK,
};

use iceoryx2::node::{
    DeadNodeView, NodeCleanupFailure, NodeDetails, NodeListFailure, NodeView, NodeWaitFailure,
};
use iceoryx2::prelude::*;
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_bb_elementary_traits::AsCStr;
use iceoryx2_ffi_macros::iceoryx2_ffi;
use iceoryx2_ffi_macros::CStrRepr;

use alloc::ffi::CString;
use core::ffi::{c_char, c_int};
use core::mem::ManuallyDrop;
use core::time::Duration;

use super::{iox2_config_h_ref, iox2_node_id_h_ref, iox2_node_id_ptr, iox2_signal_handling_mode_e};

// BEGIN type definition

/// The failures that can occur when a list of node states is created with [`iox2_node_list()`].
#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_node_list_failure_e {
    /// A list of all Nodes could not be created since the process does not have sufficient permissions.
    INSUFFICIENT_PERMISSIONS = IOX2_OK as isize + 1,
    /// The process received an interrupt signal while acquiring the list of all Nodes.
    INTERRUPT,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
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
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_node_wait_failure_e {
    INTERRUPT = IOX2_OK as isize + 1,
    TERMINATION_REQUEST,
}

impl IntoCInt for NodeWaitFailure {
    fn into_c_int(self) -> c_int {
        (match self {
            NodeWaitFailure::TerminationRequest => iox2_node_wait_failure_e::TERMINATION_REQUEST,
            NodeWaitFailure::Interrupt => iox2_node_wait_failure_e::INTERRUPT,
        }) as c_int
    }
}

/// Failures of [`iox2_dead_node_remove_stale_resources()`] that occur when the stale resources of
/// a dead node are removed.
#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_node_cleanup_failure_e {
    /// The process received an interrupt signal while cleaning up all stale resources of a dead node.
    INTERRUPT = IOX2_OK as isize + 1,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    INTERNAL_ERROR,
    /// The stale resources of a dead node could not be removed since the process does not have sufficient permissions.
    INSUFFICIENT_PERMISSIONS,
    /// Trying to cleanup resources from a dead node which was using a different iceoryx2 version.
    VERSION_MISMATCH,
}

impl IntoCInt for NodeCleanupFailure {
    fn into_c_int(self) -> c_int {
        (match self {
            NodeCleanupFailure::Interrupt => iox2_node_cleanup_failure_e::INTERRUPT,
            NodeCleanupFailure::InternalError => iox2_node_cleanup_failure_e::INTERNAL_ERROR,
            NodeCleanupFailure::InsufficientPermissions => {
                iox2_node_cleanup_failure_e::INSUFFICIENT_PERMISSIONS
            }
            NodeCleanupFailure::VersionMismatch => iox2_node_cleanup_failure_e::VERSION_MISMATCH,
        }) as c_int
    }
}

pub(super) union NodeUnion {
    ipc: ManuallyDrop<Node<crate::IpcService>>,
    local: ManuallyDrop<Node<crate::LocalService>>,
}

impl NodeUnion {
    pub(super) fn new_ipc(node: Node<crate::IpcService>) -> Self {
        Self {
            ipc: ManuallyDrop::new(node),
        }
    }
    pub(super) fn new_local(node: Node<crate::LocalService>) -> Self {
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
/// The non-owning handle for `iox2_node_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_node_h_ref = *const iox2_node_h;

impl AssertNonNullHandle for iox2_node_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_node_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_node_h {
    type Target = *mut iox2_node_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_node_h_ref {
    type Target = *mut iox2_node_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
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
    *const c_char,
    iox2_node_name_ptr,
    iox2_config_ptr,
    iox2_callback_context,
) -> iox2_callback_progression_e;

// END type definition

// BEGIN C API

/// Returns a string representation of the [`iox2_node_list_failure_e`] error code.
///
/// # Arguments
///
/// * `error` - The error value for which a description should be returned
///
/// Returns a pointer to a null-terminated string containing the error description.
///
/// # Safety
///
/// * The returned pointer is valid as long as the program runs and must not be modified or freed
#[no_mangle]
pub unsafe extern "C" fn iox2_node_list_failure_string(
    error: iox2_node_list_failure_e,
) -> *const c_char {
    error.as_const_cstr().as_ptr() as *const c_char
}

/// Returns a string representation of the [`iox2_node_wait_failure_e`] error code.
///
/// # Arguments
///
/// * `error` - The error value for which a description should be returned
///
/// # Returns
///
/// A pointer to a null-terminated string containing the error message.
/// The string is stored in the .rodata section of the binary.
///
/// # Safety
///
/// * The returned pointer is valid as long as the program runs and must not be modified or freed
#[no_mangle]
pub unsafe extern "C" fn iox2_node_wait_failure_string(
    error: iox2_node_wait_failure_e,
) -> *const c_char {
    error.as_const_cstr().as_ptr() as *const c_char
}

/// Returns the [`iox2_node_name_ptr`](crate::iox2_node_name_ptr), an immutable pointer to the node name.
///
/// # Safety
///
/// * The `node_handle` must be valid and obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_name(node_handle: iox2_node_h_ref) -> iox2_node_name_ptr {
    node_handle.assert_non_null();

    let node = &mut *node_handle.as_type();

    match node.service_type {
        iox2_service_type_e::IPC => node.value.as_ref().ipc.name(),
        iox2_service_type_e::LOCAL => node.value.as_ref().local.name(),
    }
}

/// Wait until the provided cycle time has passed and returns a [`iox2_node_wait_failure_e`] enum containing the event that
/// has occurred.
///
/// # Safety
///
/// * The `node_handle` must be valid and obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_wait(
    node_handle: iox2_node_h_ref,
    cycle_time_sec: u64,
    cycle_time_nsec: u32,
) -> c_int {
    node_handle.assert_non_null();

    let node = &mut *node_handle.as_type();
    let cycle_time =
        Duration::from_secs(cycle_time_sec) + Duration::from_nanos(cycle_time_nsec as u64);

    let result = match node.service_type {
        iox2_service_type_e::IPC => node.value.as_ref().ipc.wait(cycle_time),
        iox2_service_type_e::LOCAL => node.value.as_ref().local.wait(cycle_time),
    };

    match result {
        Ok(()) => IOX2_OK,
        Err(e) => e.into_c_int(),
    }
}

/// Returns the [`iox2_config_ptr`](crate::iox2_config_ptr), an immutable pointer to the config.
///
/// # Safety
///
/// * The `node_handle` must be valid and obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_config(node_handle: iox2_node_h_ref) -> iox2_config_ptr {
    node_handle.assert_non_null();

    let node = &mut *node_handle.as_type();

    match node.service_type {
        iox2_service_type_e::IPC => node.value.as_ref().ipc.config(),
        iox2_service_type_e::LOCAL => node.value.as_ref().local.config(),
    }
}

/// Returns the [`iox2_signal_handling_mode_e`] with which the node was created.
///
/// # Safety
///
/// * The `node_handle` must be valid and obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_signal_handling_mode(
    node_handle: iox2_node_h_ref,
) -> iox2_signal_handling_mode_e {
    node_handle.assert_non_null();

    let node = &mut *node_handle.as_type();

    match node.service_type {
        iox2_service_type_e::IPC => node.value.as_ref().ipc.signal_handling_mode().into(),
        iox2_service_type_e::LOCAL => node.value.as_ref().local.signal_handling_mode().into(),
    }
}

/// Returns the [`iox2_node_id_ptr`](crate::iox2_node_id_ptr), an immutable pointer to the node id.
///
/// # Safety
///
/// * The `node_handle` must be valid and obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_id(
    node_handle: iox2_node_h_ref,
    service_type: iox2_service_type_e,
) -> iox2_node_id_ptr {
    node_handle.assert_non_null();

    let node = &mut *node_handle.as_type();
    match service_type {
        iox2_service_type_e::IPC => node.value.as_ref().ipc.id(),
        iox2_service_type_e::LOCAL => node.value.as_ref().local.id(),
    }
}

/// Removes all stale resources of a dead node under a provided config.
///
/// Returns [`IOX2_OK`] on success, otherwise [`iox2_node_cleanup_failure_e`].
///
/// # Safety
///
/// * The `node_handle` must be valid and obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
/// * The `node_id` must be valid
/// * The `config` must be valid
/// * `has_success` must point to a valid memory location
#[no_mangle]
pub unsafe extern "C" fn iox2_dead_node_remove_stale_resources(
    service_type: iox2_service_type_e,
    node_id: iox2_node_id_h_ref,
    config: iox2_config_h_ref,
    has_success: *mut bool,
) -> c_int {
    node_id.assert_non_null();
    config.assert_non_null();
    debug_assert!(!has_success.is_null());

    let node_id = (*node_id.as_type()).value.as_ref();
    let config = (*config.as_type()).value.as_ref();

    let result = match service_type {
        iox2_service_type_e::IPC => {
            DeadNodeView::<crate::IpcService>::__internal_remove_stale_resources(
                *node_id,
                NodeDetails::__internal_new(&None, &config.value),
            )
        }
        iox2_service_type_e::LOCAL => {
            DeadNodeView::<crate::LocalService>::__internal_remove_stale_resources(
                *node_id,
                NodeDetails::__internal_new(&None, &config.value),
            )
        }
    };

    match result {
        Ok(v) => {
            *has_success = v;
            IOX2_OK
        }
        Err(e) => e.into_c_int(),
    }
}

pub(crate) fn iox2_node_list_impl<S: Service>(
    node_state: &NodeState<S>,
    callback: iox2_node_list_callback,
    callback_ctx: iox2_callback_context,
) -> CallbackProgression {
    let unknown_executable = CString::new("unknown_executable").unwrap();
    match node_state {
        NodeState::Alive(alive_node_view) => {
            let (executable, node_name, config) = alive_node_view
                .details()
                .as_ref()
                .map(|view| {
                    (
                        CString::new(view.executable().as_bytes()).unwrap(),
                        view.name() as _,
                        view.config() as _,
                    )
                })
                .unwrap_or((unknown_executable, core::ptr::null(), core::ptr::null()));
            callback(
                iox2_node_state_e::ALIVE,
                alive_node_view.id(),
                executable.as_bytes_with_nul().as_ptr().cast(),
                node_name,
                config,
                callback_ctx,
            )
            .into()
        }
        NodeState::Dead(dead_node_view) => {
            let (executable, node_name, config) = dead_node_view
                .details()
                .as_ref()
                .map(|view| {
                    (
                        CString::new(view.executable().as_bytes()).unwrap(),
                        view.name() as _,
                        view.config() as _,
                    )
                })
                .unwrap_or((unknown_executable, core::ptr::null(), core::ptr::null()));
            callback(
                iox2_node_state_e::DEAD,
                dead_node_view.id(),
                executable.as_bytes_with_nul().as_ptr().cast(),
                node_name,
                config,
                callback_ctx,
            )
            .into()
        }
        NodeState::Inaccessible(ref node_id) => callback(
            iox2_node_state_e::INACCESSIBLE,
            node_id,
            unknown_executable.as_bytes_with_nul().as_ptr().cast(),
            core::ptr::null(),
            core::ptr::null(),
            callback_ctx,
        )
        .into(),
        NodeState::Undefined(ref node_id) => callback(
            iox2_node_state_e::UNDEFINED,
            node_id,
            unknown_executable.as_bytes_with_nul().as_ptr().cast(),
            core::ptr::null(),
            core::ptr::null(),
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
        iox2_service_type_e::IPC => Node::<crate::IpcService>::list(config, |node_state| {
            iox2_node_list_impl(&node_state, callback, callback_ctx)
        }),
        iox2_service_type_e::LOCAL => Node::<crate::LocalService>::list(config, |node_state| {
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
/// * `node_handle` - Must be a valid [`iox2_node_h_ref`] obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)
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
    node_handle: iox2_node_h_ref,
    service_builder_struct_ptr: *mut iox2_service_builder_t,
    service_name_ptr: iox2_service_name_ptr,
) -> iox2_service_builder_h {
    node_handle.assert_non_null();
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
    node_handle.assert_non_null();

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
