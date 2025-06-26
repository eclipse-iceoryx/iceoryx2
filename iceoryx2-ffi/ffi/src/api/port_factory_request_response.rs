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

// BEGIN types definition

use core::{
    ffi::{c_char, c_int},
    mem::ManuallyDrop,
};
use iceoryx2::service::dynamic_config::request_response::ServerDetails;
use iceoryx2::service::{
    dynamic_config::request_response::ClientDetails, port_factory::request_response::PortFactory,
};
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use crate::{
    api::{IntoCInt, PortFactoryClientBuilderUnion, PortFactoryServerBuilderUnion},
    iox2_node_list_impl, IOX2_OK,
};

use super::{
    iox2_attribute_set_ptr, iox2_callback_context, iox2_callback_progression_e,
    iox2_client_details_ptr, iox2_node_list_callback, iox2_port_factory_client_builder_h,
    iox2_port_factory_client_builder_t, iox2_port_factory_server_builder_h,
    iox2_port_factory_server_builder_t, iox2_server_details_ptr, iox2_service_name_ptr,
    iox2_service_type_e, iox2_static_config_request_response_t, AssertNonNullHandle, HandleToType,
    PayloadFfi, UserHeaderFfi,
};

// BEGIN types definition
pub(super) union PortFactoryRequestResponseUnion {
    ipc: ManuallyDrop<
        PortFactory<crate::IpcService, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
    >,
    local: ManuallyDrop<
        PortFactory<crate::LocalService, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
    >,
}

impl PortFactoryRequestResponseUnion {
    pub(super) fn new_ipc(
        port_factory: PortFactory<
            crate::IpcService,
            PayloadFfi,
            UserHeaderFfi,
            PayloadFfi,
            UserHeaderFfi,
        >,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(port_factory),
        }
    }
    pub(super) fn new_local(
        port_factory: PortFactory<
            crate::LocalService,
            PayloadFfi,
            UserHeaderFfi,
            PayloadFfi,
            UserHeaderFfi,
        >,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(port_factory),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<PortFactoryRequestResponseUnion>
pub struct iox2_port_factory_request_response_storage_t {
    internal: [u8; 1656], // magic number obtained with size_of::<Option<PortFactoryRequestResponseUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(PortFactoryRequestResponseUnion)]
pub struct iox2_port_factory_request_response_t {
    service_type: iox2_service_type_e,
    value: iox2_port_factory_request_response_storage_t,
    deleter: fn(*mut iox2_port_factory_request_response_t),
}

impl iox2_port_factory_request_response_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: PortFactoryRequestResponseUnion,
        deleter: fn(*mut iox2_port_factory_request_response_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_port_factory_request_response_h_t;
/// The owning handle for `iox2_port_factory_request_response_t`. Passing the handle to a function transfers the ownership.
pub type iox2_port_factory_request_response_h = *mut iox2_port_factory_request_response_h_t;
/// The non-owning handle for `iox2_port_factory_request_response_t`. Passing the handle to a function does not transfer the ownership.
pub type iox2_port_factory_request_response_h_ref = *const iox2_port_factory_request_response_h;

impl AssertNonNullHandle for iox2_port_factory_request_response_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_port_factory_request_response_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_port_factory_request_response_h {
    type Target = *mut iox2_port_factory_request_response_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_port_factory_request_response_h_ref {
    type Target = *mut iox2_port_factory_request_response_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

/// The callback for [`iox2_port_factory_request_response_dynamic_config_list_servers()`]
///
/// # Arguments
///
/// * [`iox2_callback_context`] -> provided by the user and can be `NULL`
/// * [`iox2_server_details_ptr`] -> a pointer to the details struct of the port
///
/// Returns a [`iox2_callback_progression_e`](crate::iox2_callback_progression_e)
pub type iox2_list_servers_callback =
    extern "C" fn(iox2_callback_context, iox2_server_details_ptr) -> iox2_callback_progression_e;

/// The callback for [`iox2_port_factory_request_response_dynamic_config_list_clients()`]
///
/// # Arguments
///
/// * [`iox2_callback_context`] -> provided by the user and can be `NULL`
/// * [`iox2_client_details_ptr`] -> a pointer to the details struct of the port
///
/// Returns a [`iox2_callback_progression_e`](crate::iox2_callback_progression_e)
pub type iox2_list_clients_callback =
    extern "C" fn(iox2_callback_context, iox2_client_details_ptr) -> iox2_callback_progression_e;

// END type definition

// BEGIN C API

/// Instantiates a [`iox2_port_factory_server_builder_h`] to build a server.
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_request_response_h_ref`] obtained
///   by e.g. [`iox2_service_builder_request_response_open_or_create`](crate::iox2_service_builder_request_response_open_or_create).
/// * `builder_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_port_factory_server_builder_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
///
/// Returns the `iox2_port_factory_server_builder_h` handle for the server builder.
///
/// # Safety
///
/// * The `port_factory_handle` is still valid after the return of this function and can be used in another function call.
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_request_response_server_builder(
    port_factory_handle: iox2_port_factory_request_response_h_ref,
    builder_struct_ptr: *mut iox2_port_factory_server_builder_t,
) -> iox2_port_factory_server_builder_h {
    port_factory_handle.assert_non_null();

    let mut builder_struct_ptr = builder_struct_ptr;
    fn no_op(_: *mut iox2_port_factory_server_builder_t) {}
    let mut deleter: fn(*mut iox2_port_factory_server_builder_t) = no_op;
    if builder_struct_ptr.is_null() {
        builder_struct_ptr = iox2_port_factory_server_builder_t::alloc();
        deleter = iox2_port_factory_server_builder_t::dealloc;
    }
    debug_assert!(!builder_struct_ptr.is_null());

    let port_factory = &mut *port_factory_handle.as_type();
    match port_factory.service_type {
        iox2_service_type_e::IPC => {
            let server_builder = port_factory.value.as_ref().ipc.server_builder();
            (*builder_struct_ptr).init(
                port_factory.service_type,
                PortFactoryServerBuilderUnion::new_ipc(server_builder),
                deleter,
            );
        }
        iox2_service_type_e::LOCAL => {
            let server_builder = port_factory.value.as_ref().local.server_builder();
            (*builder_struct_ptr).init(
                port_factory.service_type,
                PortFactoryServerBuilderUnion::new_local(server_builder),
                deleter,
            );
        }
    };

    (*builder_struct_ptr).as_handle()
}

/// Instantiates a [`iox2_port_factory_client_builder_h`] to build a client.
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_request_response_h_ref`] obtained
///   by e.g. [`iox2_service_builder_request_response_open_or_create`](crate::iox2_service_builder_request_response_open_or_create).
/// * `builder_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_port_factory_client_builder_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
///
/// Returns the `iox2_port_factory_client_builder_h` handle for the client builder.
///
/// # Safety
///
/// * The `port_factory_handle` is still valid after the return of this function and can be used in another function call.
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_request_response_client_builder(
    port_factory_handle: iox2_port_factory_request_response_h_ref,
    builder_struct_ptr: *mut iox2_port_factory_client_builder_t,
) -> iox2_port_factory_client_builder_h {
    port_factory_handle.assert_non_null();

    let mut builder_struct_ptr = builder_struct_ptr;
    fn no_op(_: *mut iox2_port_factory_client_builder_t) {}
    let mut deleter: fn(*mut iox2_port_factory_client_builder_t) = no_op;
    if builder_struct_ptr.is_null() {
        builder_struct_ptr = iox2_port_factory_client_builder_t::alloc();
        deleter = iox2_port_factory_client_builder_t::dealloc;
    }
    debug_assert!(!builder_struct_ptr.is_null());

    let port_factory = &mut *port_factory_handle.as_type();
    match port_factory.service_type {
        iox2_service_type_e::IPC => {
            let client_builder = port_factory.value.as_ref().ipc.client_builder();
            (*builder_struct_ptr).init(
                port_factory.service_type,
                PortFactoryClientBuilderUnion::new_ipc(client_builder),
                deleter,
            );
        }
        iox2_service_type_e::LOCAL => {
            let client_builder = port_factory.value.as_ref().local.client_builder();
            (*builder_struct_ptr).init(
                port_factory.service_type,
                PortFactoryClientBuilderUnion::new_local(client_builder),
                deleter,
            );
        }
    };

    (*builder_struct_ptr).as_handle()
}

/// Returns the services attributes.
///
/// # Safety
///
/// * The `port_factory_handle` is still valid after the return of this function and can be used in another function call.
/// * The `port_factory_handle` must live longer than the returned `iox2_attribute_set_h_ref`.
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_request_response_attributes(
    port_factory_handle: iox2_port_factory_request_response_h_ref,
) -> iox2_attribute_set_ptr {
    use iceoryx2::prelude::PortFactory;

    port_factory_handle.assert_non_null();

    let port_factory = &mut *port_factory_handle.as_type();
    match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory.value.as_ref().ipc.attributes(),
        iox2_service_type_e::LOCAL => port_factory.value.as_ref().local.attributes(),
    }
}

/// Set the values in the provided [`iox2_static_config_request_response_t`] pointer.
///
/// # Safety
///
/// * The `port_factory_handle` must be valid and obtained by [`iox2_service_builder_request_response_open`](crate::iox2_service_builder_request_response_open) or
///   [`iox2_service_builder_request_response_open_or_create`](crate::iox2_service_builder_request_response_open_or_create)!
/// * The `static_config` must be a valid pointer and non-null.
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_request_response_static_config(
    port_factory_handle: iox2_port_factory_request_response_h_ref,
    static_config: *mut iox2_static_config_request_response_t,
) {
    port_factory_handle.assert_non_null();
    debug_assert!(!static_config.is_null());

    let port_factory = &mut *port_factory_handle.as_type();

    use iceoryx2::prelude::PortFactory;
    let config = match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory.value.as_ref().ipc.static_config(),
        iox2_service_type_e::LOCAL => port_factory.value.as_ref().local.static_config(),
    };

    *static_config = config.into();
}

/// Returns how many server ports are currently connected.
///
/// # Safety
///
/// * The `handle` must be valid and obtained by [`iox2_service_builder_request_response_open`](crate::iox2_service_builder_request_response_open) or
///   [`iox2_service_builder_request_response_open_or_create`](crate::iox2_service_builder_request_response_open_or_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_request_response_dynamic_config_number_of_servers(
    handle: iox2_port_factory_request_response_h_ref,
) -> usize {
    handle.assert_non_null();

    let port_factory = &mut *handle.as_type();

    use iceoryx2::prelude::PortFactory;
    match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory
            .value
            .as_ref()
            .ipc
            .dynamic_config()
            .number_of_servers(),
        iox2_service_type_e::LOCAL => port_factory
            .value
            .as_ref()
            .local
            .dynamic_config()
            .number_of_servers(),
    }
}

/// Returns how many client ports are currently connected.
///
/// # Safety
///
/// * The `handle` must be valid and obtained by [`iox2_service_builder_request_response_open`](crate::iox2_service_builder_request_response_open) or
///   [`iox2_service_builder_request_response_open_or_create`](crate::iox2_service_builder_request_response_open_or_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_request_response_dynamic_config_number_of_clients(
    handle: iox2_port_factory_request_response_h_ref,
) -> usize {
    handle.assert_non_null();

    let port_factory = &mut *handle.as_type();

    use iceoryx2::prelude::PortFactory;
    match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory
            .value
            .as_ref()
            .ipc
            .dynamic_config()
            .number_of_clients(),
        iox2_service_type_e::LOCAL => port_factory
            .value
            .as_ref()
            .local
            .dynamic_config()
            .number_of_clients(),
    }
}

/// Calls the callback repeatedly with an [`iox2_node_state_e`](crate::api::iox2_node_state_e),
/// [`iox2_node_id_ptr`](crate::api::iox2_node_id_ptr),
/// [´iox2_node_name_ptr´](crate::api::iox2_node_name_ptr) and
/// [`iox2_config_ptr`](crate::api::iox2_config_ptr) for all
/// [`Node`](iceoryx2::node::Node)s that
/// have opened the service.
///
/// Returns IOX2_OK on success, an
/// [`iox2_node_list_failure_e`](crate::api::iox2_node_list_failure_e) otherwise.
///
/// # Safety
///
/// * The `handle` must be valid and obtained by [`iox2_service_builder_request_response_open`](crate::iox2_service_builder_request_response_open) or
///   [`iox2_service_builder_request_response_open_or_create`](crate::iox2_service_builder_request_response_open_or_create)!
/// * `callback` - A valid callback with [`iox2_node_list_callback`} signature
/// * `callback_ctx` - An optional callback context [`iox2_callback_context`} to e.g. store information across callback iterations
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_request_response_nodes(
    handle: iox2_port_factory_request_response_h_ref,
    callback: iox2_node_list_callback,
    callback_ctx: iox2_callback_context,
) -> c_int {
    use iceoryx2::prelude::PortFactory;

    handle.assert_non_null();

    let port_factory = &mut *handle.as_type();

    let list_result = match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory
            .value
            .as_ref()
            .ipc
            .nodes(|node_state| iox2_node_list_impl(&node_state, callback, callback_ctx)),
        iox2_service_type_e::LOCAL => port_factory
            .value
            .as_ref()
            .local
            .nodes(|node_state| iox2_node_list_impl(&node_state, callback, callback_ctx)),
    };

    match list_result {
        Ok(_) => IOX2_OK,
        Err(e) => e.into_c_int(),
    }
}

/// Returns the [`iox2_service_name_ptr`], an immutable pointer to the service name.
///
/// # Safety
///
/// * The `handle` must be valid and obtained by [`iox2_service_builder_request_response_open`](crate::iox2_service_builder_request_response_open) or
///   [`iox2_service_builder_request_response_open_or_create`](crate::iox2_service_builder_request_response_open_or_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_request_response_service_name(
    handle: iox2_port_factory_request_response_h_ref,
) -> iox2_service_name_ptr {
    use iceoryx2::prelude::PortFactory;

    handle.assert_non_null();

    let port_factory = &mut *handle.as_type();

    match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory.value.as_ref().ipc.name(),
        iox2_service_type_e::LOCAL => port_factory.value.as_ref().local.name(),
    }
}

/// Stores the service id in the provided buffer
///
/// # Safety
///
/// * The `handle` must be valid and obtained by [`iox2_service_builder_request_response_open`](crate::iox2_service_builder_request_response_open) or
///   [`iox2_service_builder_request_response_open_or_create`](crate::iox2_service_builder_request_response_open_or_create)!
/// * `buffer` must be non-zero and point to a valid memory location
/// * `buffer_len` must define the actual size of the memory location `buffer` is pointing to
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_request_response_service_id(
    handle: iox2_port_factory_request_response_h_ref,
    buffer: *mut c_char,
    buffer_len: usize,
) {
    use iceoryx2::prelude::PortFactory;

    debug_assert!(!buffer.is_null());
    handle.assert_non_null();

    let port_factory = &mut *handle.as_type();
    let service_id = match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory.value.as_ref().ipc.service_id(),
        iox2_service_type_e::LOCAL => port_factory.value.as_ref().local.service_id(),
    };

    let len = buffer_len.min(service_id.as_str().len());
    core::ptr::copy_nonoverlapping(service_id.as_str().as_ptr(), buffer.cast(), len);
    buffer.add(len).write(0);
}

/// Calls the callback repeatedly for every connected [`iox2_server_h`](crate::iox2_server_h)
/// and provides all communcation details with a [`iox2_server_details_ptr`].
///
/// # Safety
///
/// * [`iox2_server_details_ptr`] - Provides a view to the server details. Data must not be
///   accessed outside of the callback.
/// * `callback` - A valid callback with [`iox2_list_servers_callback`] signature
/// * `callback_ctx` - An optional callback context [`iox2_callback_context`] to e.g. store
///   information across callback iterations. Must be either `NULL` or point to a valid memory
///   location
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_request_response_dynamic_config_list_servers(
    handle: iox2_port_factory_request_response_h_ref,
    callback: iox2_list_servers_callback,
    callback_ctx: iox2_callback_context,
) {
    handle.assert_non_null();
    use iceoryx2::prelude::PortFactory;

    let port_factory = &mut *handle.as_type();
    let callback_tr = |server: &ServerDetails| callback(callback_ctx, server).into();
    match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory
            .value
            .as_ref()
            .ipc
            .dynamic_config()
            .list_servers(callback_tr),
        iox2_service_type_e::LOCAL => port_factory
            .value
            .as_ref()
            .local
            .dynamic_config()
            .list_servers(callback_tr),
    };
}

/// Calls the callback repeatedly with for every connected [`iox2_client_h`](crate::iox2_client_h)
/// and provides all communcation details with a [`iox2_client_details_ptr`].
///
/// # Safety
///
/// * [`iox2_client_details_ptr`] - Provides a view to the client details. Data must not be
///   accessed outside of the callback.
/// * `callback` - A valid callback with [`iox2_list_clients_callback`] signature
/// * `callback_ctx` - An optional callback context [`iox2_callback_context`] to e.g. store
///   information across callback iterations. Must be either `NULL` or point to a valid memory
///   location
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_request_response_dynamic_config_list_clients(
    handle: iox2_port_factory_request_response_h_ref,
    callback: iox2_list_clients_callback,
    callback_ctx: iox2_callback_context,
) {
    handle.assert_non_null();
    use iceoryx2::prelude::PortFactory;

    let port_factory = &mut *handle.as_type();
    let callback_tr = |client: &ClientDetails| callback(callback_ctx, client).into();
    match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory
            .value
            .as_ref()
            .ipc
            .dynamic_config()
            .list_clients(callback_tr),
        iox2_service_type_e::LOCAL => port_factory
            .value
            .as_ref()
            .local
            .dynamic_config()
            .list_clients(callback_tr),
    };
}

/// This function needs to be called to destroy the port factory!
///
/// # Arguments
///
/// * `port_factory_handle` - A valid [`iox2_port_factory_request_response_h`]
///
/// # Safety
///
/// * The `port_factory_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_port_factory_request_response_t`] can be re-used with a call to
///   [`iox2_service_builder_request_response_open_or_create`](crate::iox2_service_builder_request_response_open_or_create) or
///   [`iox2_service_builder_request_response_open`](crate::iox2_service_builder_request_response_open)!
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_request_response_drop(
    port_factory_handle: iox2_port_factory_request_response_h,
) {
    debug_assert!(!port_factory_handle.is_null());

    let port_factory = &mut *port_factory_handle.as_type();

    match port_factory.service_type {
        iox2_service_type_e::IPC => {
            ManuallyDrop::drop(&mut port_factory.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            ManuallyDrop::drop(&mut port_factory.value.as_mut().local);
        }
    }
    (port_factory.deleter)(port_factory);
}
// END C API
