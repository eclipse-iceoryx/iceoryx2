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

use crate::api::{ActiveRequestUnion, IntoCInt};
use crate::IOX2_OK;

use super::{
    c_size_t, iox2_active_request_h, iox2_active_request_t, iox2_service_type_e,
    iox2_unique_server_id_h, iox2_unique_server_id_t, AssertNonNullHandle, HandleToType,
};
use super::{PayloadFfi, UserHeaderFfi};
use core::ffi::c_int;
use core::mem::ManuallyDrop;
use iceoryx2::port::server::Server;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

// BEGIN types definition
pub(super) union ServerUnion {
    ipc: ManuallyDrop<
        Server<crate::IpcService, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
    >,
    local: ManuallyDrop<
        Server<crate::LocalService, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
    >,
}

impl ServerUnion {
    pub(super) fn new_ipc(
        server: Server<crate::IpcService, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(server),
        }
    }
    pub(super) fn new_local(
        server: Server<crate::LocalService, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(server),
        }
    }
}

#[repr(C)]
#[repr(align(16))] // alignment of Option<ServerUnion>
pub struct iox2_server_storage_t {
    internal: [u8; 248], // magic number obtained with size_of::<Option<ServerUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(ServerUnion)]
pub struct iox2_server_t {
    service_type: iox2_service_type_e,
    value: iox2_server_storage_t,
    deleter: fn(*mut iox2_server_t),
}

impl iox2_server_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: ServerUnion,
        deleter: fn(*mut iox2_server_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_server_h_t;
/// The owning handle for `iox2_server_t`. Passing the handle to a function transfers the ownership.
pub type iox2_server_h = *mut iox2_server_h_t;
/// The non-owning handle for `iox2_server_t`. Passing the handle to a function does not transfer the ownership.
pub type iox2_server_h_ref = *const iox2_server_h;

impl AssertNonNullHandle for iox2_server_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_server_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_server_h {
    type Target = *mut iox2_server_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_server_h_ref {
    type Target = *mut iox2_server_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END types definition

// BEGIN C API

/// Returns the unique port id of the server.
///
/// # Arguments
///
/// * `handle` obtained by [`iox2_port_factory_server_builder_create`](crate::iox2_port_factory_server_builder_create)
/// * `id_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_unique_server_id_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
/// * `id_handle_ptr` valid pointer to a [`iox2_unique_server_id_h`].
///
/// # Safety
///
/// * `handle` is valid, non-null and was obtained via [`iox2_port_factory_server_builder_create`](crate::iox2_port_factory_server_builder_create)
/// * `id_handle_ptr` is valid and non-null
#[no_mangle]
pub unsafe extern "C" fn iox2_server_id(
    handle: iox2_server_h_ref,
    id_struct_ptr: *mut iox2_unique_server_id_t,
    id_handle_ptr: *mut iox2_unique_server_id_h,
) {
    handle.assert_non_null();
    debug_assert!(!id_handle_ptr.is_null());

    fn no_op(_: *mut iox2_unique_server_id_t) {}
    let mut deleter: fn(*mut iox2_unique_server_id_t) = no_op;
    let mut storage_ptr = id_struct_ptr;
    if id_struct_ptr.is_null() {
        deleter = iox2_unique_server_id_t::dealloc;
        storage_ptr = iox2_unique_server_id_t::alloc();
    }
    debug_assert!(!storage_ptr.is_null());

    let server = &mut *handle.as_type();

    let id = match server.service_type {
        iox2_service_type_e::IPC => server.value.as_mut().ipc.id(),
        iox2_service_type_e::LOCAL => server.value.as_mut().local.id(),
    };

    (*storage_ptr).init(id, deleter);
    *id_handle_ptr = (*storage_ptr).as_handle();
}

/// Returns true when the server has requests that can be acquired with [`iox2_server_receive`], otherwise false.
///
/// # Arguments
///
/// * `handle` - Must be a valid [`iox2_server_h_ref`]
///   obtained by [`iox2_port_factory_subscriber_builder_create`](crate::iox2_port_factory_subscriber_builder_create).
/// * `result_ptr` - A non-null pointer to a bool that will contain the result.
///
/// Returns IOX2_OK on success, an [`iox2_connection_failure_e`](crate::iox2_connection_failure_e) otherwise.
/// Attention, an empty server queue is not an error and even with IOX2_OK it is possible to get a NULL in `request_handle_ptr`.
///
/// # Safety
///
/// * The `handle` is still valid after the return of this function and can be use in another function call.
/// * The `result_ptr` is pointing to a valid bool.
#[no_mangle]
pub unsafe extern "C" fn iox2_server_has_requests(
    handle: iox2_server_h_ref,
    result_ptr: *mut bool,
) -> c_int {
    handle.assert_non_null();
    debug_assert!(!result_ptr.is_null());

    let server = &mut *handle.as_type();

    match server.service_type {
        iox2_service_type_e::IPC => match server.value.as_ref().ipc.has_requests() {
            Ok(v) => {
                *result_ptr = v;
                IOX2_OK
            }
            Err(error) => error.into_c_int(),
        },
        iox2_service_type_e::LOCAL => match server.value.as_ref().local.has_requests() {
            Ok(v) => {
                *result_ptr = v;
                IOX2_OK
            }
            Err(error) => error.into_c_int(),
        },
    }
}

/// Returns the initial max slice len of the server. In the dynamic memory case, slice lenght might
/// increase over time.
///
/// # Safety
///
/// * `handle` - Must be a valid [`iox2_server_h_ref`]
///   obtained by [`iox2_port_factory_server_builder_create`](crate::iox2_port_factory_server_builder_create).
#[no_mangle]
pub unsafe extern "C" fn iox2_server_initial_max_slice_len(handle: iox2_server_h_ref) -> c_size_t {
    handle.assert_non_null();

    let server = &mut *handle.as_type();

    match server.service_type {
        iox2_service_type_e::IPC => server.value.as_ref().ipc.initial_max_slice_len(),
        iox2_service_type_e::LOCAL => server.value.as_ref().local.initial_max_slice_len(),
    }
}

/// Takes a request ouf of the server queue.
///
/// # Arguments
///
/// * `server_handle` - Must be a valid [`iox2_server_h_ref`]
///   obtained by [`iox2_port_factory_server_builder_create`](crate::iox2_port_factory_server_builder_create).
/// * `active_request_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_active_request_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
/// * `active_request_handle_ptr` - An uninitialized or dangling [`iox2_active_request_h`] handle
///   which will be initialized by this function call if a request is obtained, otherwise it will be
///   set to NULL.
///
/// Returns IOX2_OK on success, an [`iox2_receive_error_e`](crate::iox2_receive_error_e) otherwise.
/// Attention, an empty server queue is not an error and even with IOX2_OK it is possible to get a NULL in `active_request_handle_ptr`.
///
/// # Safety
///
/// * The `server_handle` is still valid after the return of this function and can be used in another function call.
/// * The `active_request_handle_ptr` is pointing to a valid [`iox2_active_request_h`].
#[no_mangle]
pub unsafe extern "C" fn iox2_server_receive(
    server_handle: iox2_server_h_ref,
    active_request_struct_ptr: *mut iox2_active_request_t,
    active_request_handle_ptr: *mut iox2_active_request_h,
) -> c_int {
    server_handle.assert_non_null();
    debug_assert!(!active_request_handle_ptr.is_null());

    *active_request_handle_ptr = core::ptr::null_mut();

    let init_active_request_struct_ptr = |active_request_struct_ptr: *mut iox2_active_request_t| {
        let mut active_request_struct_ptr = active_request_struct_ptr;
        fn no_op(_: *mut iox2_active_request_t) {}
        let mut deleter: fn(*mut iox2_active_request_t) = no_op;
        if active_request_struct_ptr.is_null() {
            active_request_struct_ptr = iox2_active_request_t::alloc();
            deleter = iox2_active_request_t::dealloc;
        }
        debug_assert!(!active_request_struct_ptr.is_null());

        (active_request_struct_ptr, deleter)
    };

    let server = &mut *server_handle.as_type();

    match server.service_type {
        iox2_service_type_e::IPC => match server.value.as_ref().ipc.receive_custom_payload() {
            Ok(Some(active_request)) => {
                let (active_request_struct_ptr, deleter) =
                    init_active_request_struct_ptr(active_request_struct_ptr);
                (*active_request_struct_ptr).init(
                    server.service_type,
                    ActiveRequestUnion::new_ipc(active_request),
                    deleter,
                );
                *active_request_handle_ptr = (*active_request_struct_ptr).as_handle();
            }
            Ok(None) => (),
            Err(error) => return error.into_c_int(),
        },
        iox2_service_type_e::LOCAL => match server.value.as_ref().local.receive_custom_payload() {
            Ok(Some(active_request)) => {
                let (active_request_struct_ptr, deleter) =
                    init_active_request_struct_ptr(active_request_struct_ptr);
                (*active_request_struct_ptr).init(
                    server.service_type,
                    ActiveRequestUnion::new_local(active_request),
                    deleter,
                );
                *active_request_handle_ptr = (*active_request_struct_ptr).as_handle();
            }
            Ok(None) => (),
            Err(error) => return error.into_c_int(),
        },
    }

    IOX2_OK
}

/// This function needs to be called to destroy the server!
///
/// # Arguments
///
/// * `handle` - A valid [`iox2_server_h`]
///
/// # Safety
///
/// * The `handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_server_t`] can be re-used with a call to
///   [`iox2_port_factory_subscriber_builder_create`](crate::iox2_port_factory_subscriber_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_server_drop(handle: iox2_server_h) {
    handle.assert_non_null();

    let server = &mut *handle.as_type();

    match server.service_type {
        iox2_service_type_e::IPC => {
            ManuallyDrop::drop(&mut server.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            ManuallyDrop::drop(&mut server.value.as_mut().local);
        }
    }
    (server.deleter)(server);
}
// END C API
