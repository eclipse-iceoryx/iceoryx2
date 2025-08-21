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

use core::ffi::c_void;
use core::mem::ManuallyDrop;
use iceoryx2::pending_response::PendingResponse;
use iceoryx2::port::client::Client;
use iceoryx2::prelude::*;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use crate::api::IntoCInt;
use crate::api::RequestMutUninitUnion;
use crate::IOX2_OK;

use super::iox2_pending_response_h;
use super::iox2_pending_response_t;
use super::iox2_request_mut_h;
use super::iox2_request_mut_t;
use super::iox2_request_send_error_e;
use super::iox2_service_type_e;
use super::iox2_unable_to_deliver_strategy_e;
use super::iox2_unique_client_id_h;
use super::iox2_unique_client_id_t;
use super::AssertNonNullHandle;
use super::HandleToType;
use super::PayloadFfi;
use super::PendingResponseUnion;
use super::UserHeaderFfi;
use core::ffi::c_int;

// BEGIN types definition
pub(super) union ClientUnion {
    ipc: ManuallyDrop<
        Client<crate::IpcService, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
    >,
    local: ManuallyDrop<
        Client<crate::LocalService, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
    >,
}

impl ClientUnion {
    pub(super) fn new_ipc(
        client: Client<crate::IpcService, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(client),
        }
    }
    pub(super) fn new_local(
        client: Client<crate::LocalService, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(client),
        }
    }
}

#[repr(C)]
#[repr(align(16))] // alignment of Option<ClientUnion>
pub struct iox2_client_storage_t {
    internal: [u8; 248], // magic number obtained with size_of::<Option<ClientUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(ClientUnion)]
pub struct iox2_client_t {
    service_type: iox2_service_type_e,
    value: iox2_client_storage_t,
    deleter: fn(*mut iox2_client_t),
}

impl iox2_client_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: ClientUnion,
        deleter: fn(*mut iox2_client_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_client_h_t;
/// The owning handle for `iox2_client_t`. Passing the handle to a function transfers the ownership.
pub type iox2_client_h = *mut iox2_client_h_t;
/// The non-owning handle for `iox2_client_t`. Passing the handle to a function does not transfer the ownership.
pub type iox2_client_h_ref = *const iox2_client_h;

impl AssertNonNullHandle for iox2_client_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_client_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_client_h {
    type Target = *mut iox2_client_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_client_h_ref {
    type Target = *mut iox2_client_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}
// END types definition

// BEGIN C API

/// Returns the strategy the client follows when a request cannot be delivered
/// since the servers buffer is full.
///
/// # Arguments
///
/// * `handle` obtained by [`iox2_port_factory_client_builder_create`](crate::iox2_port_factory_client_builder_create)
///
/// Returns [`iox2_unable_to_deliver_strategy_e`].
///
/// # Safety
///
/// * `handle` is valid and non-null
#[no_mangle]
pub unsafe extern "C" fn iox2_client_unable_to_deliver_strategy(
    handle: iox2_client_h_ref,
) -> iox2_unable_to_deliver_strategy_e {
    handle.assert_non_null();

    let client = &mut *handle.as_type();

    match client.service_type {
        iox2_service_type_e::IPC => client
            .value
            .as_mut()
            .ipc
            .unable_to_deliver_strategy()
            .into(),
        iox2_service_type_e::LOCAL => client
            .value
            .as_mut()
            .local
            .unable_to_deliver_strategy()
            .into(),
    }
}

/// Returns the initial max slice len with which the client was created.
///
/// # Arguments
///
/// * `handle` obtained by [`iox2_port_factory_client_builder_create`](crate::iox2_port_factory_client_builder_create)
///
/// # Safety
///
/// * `handle` is valid and non-null
#[no_mangle]
pub unsafe extern "C" fn iox2_client_initial_max_slice_len(handle: iox2_client_h_ref) -> c_int {
    handle.assert_non_null();

    let client = &mut *handle.as_type();
    match client.service_type {
        iox2_service_type_e::IPC => client.value.as_mut().ipc.initial_max_slice_len() as c_int,
        iox2_service_type_e::LOCAL => client.value.as_mut().local.initial_max_slice_len() as c_int,
    }
}

/// Returns the unique port id of the client.
///
/// # Arguments
///
/// * `handle` obtained by [`iox2_port_factory_client_builder_create`](crate::iox2_port_factory_client_builder_create)
/// * `id_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_unique_client_id_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
/// * `id_handle_ptr` valid pointer to a [`iox2_unique_client_id_h`].
///
/// # Safety
///
/// * `handle` is valid and non-null
/// * `id` is valid and non-null
#[no_mangle]
pub unsafe extern "C" fn iox2_client_id(
    handle: iox2_client_h_ref,
    id_struct_ptr: *mut iox2_unique_client_id_t,
    id_handle_ptr: *mut iox2_unique_client_id_h,
) {
    handle.assert_non_null();
    debug_assert!(!id_handle_ptr.is_null());

    fn no_op(_: *mut iox2_unique_client_id_t) {}
    let mut deleter: fn(*mut iox2_unique_client_id_t) = no_op;
    let mut storage_ptr = id_struct_ptr;
    if id_struct_ptr.is_null() {
        deleter = iox2_unique_client_id_t::dealloc;
        storage_ptr = iox2_unique_client_id_t::alloc();
    }
    debug_assert!(!storage_ptr.is_null());

    let client = &mut *handle.as_type();

    let id = match client.service_type {
        iox2_service_type_e::IPC => client.value.as_mut().ipc.id(),
        iox2_service_type_e::LOCAL => client.value.as_mut().local.id(),
    };

    (*storage_ptr).init(id, deleter);
    *id_handle_ptr = (*storage_ptr).as_handle();
}

/// Loans memory from the clients data segment.
///
/// # Arguments
///
/// * `client_handle` obtained by [`iox2_port_factory_client_builder_create`](crate::iox2_port_factory_client_builder_create)
/// * `request_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_request_mut_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
/// * `request_handle_ptr` - An uninitialized or dangling [`iox2_request_mut_h`] handle which will be initialized by this function call if a request is obtained, otherwise it will be set to NULL.
/// * `number_of_elements` - The number of elements to loan from the clients's payload segment
///
/// Return [`IOX2_OK`] on success, otherwise [`iox2_loan_error_e`](crate::iox2_loan_error_e).
///
/// # Safety
///
/// * `client_handle` is valid and non-null
/// * The `request_handle_ptr` is pointing to a valid [`iox2_request_mut_h`].
#[no_mangle]
pub unsafe extern "C" fn iox2_client_loan_slice_uninit(
    client_handle: iox2_client_h_ref,
    request_struct_ptr: *mut iox2_request_mut_t,
    request_handle_ptr: *mut iox2_request_mut_h,
    number_of_elements: usize,
) -> c_int {
    client_handle.assert_non_null();
    debug_assert!(!request_handle_ptr.is_null());

    *request_handle_ptr = core::ptr::null_mut();

    let init_request_struct_ptr = |request_struct_ptr: *mut iox2_request_mut_t| {
        let mut request_struct_ptr = request_struct_ptr;
        fn no_op(_: *mut iox2_request_mut_t) {}
        let mut deleter: fn(*mut iox2_request_mut_t) = no_op;
        if request_struct_ptr.is_null() {
            request_struct_ptr = iox2_request_mut_t::alloc();
            deleter = iox2_request_mut_t::dealloc;
        }
        debug_assert!(!request_struct_ptr.is_null());

        (request_struct_ptr, deleter)
    };

    let client = &mut *client_handle.as_type();

    match client.service_type {
        iox2_service_type_e::IPC => match client
            .value
            .as_ref()
            .ipc
            .loan_custom_payload(number_of_elements)
        {
            Ok(request) => {
                let (request_struct_ptr, deleter) = init_request_struct_ptr(request_struct_ptr);
                (*request_struct_ptr).init(
                    client.service_type,
                    RequestMutUninitUnion::new_ipc(request),
                    deleter,
                );
                *request_handle_ptr = (*request_struct_ptr).as_handle();
                IOX2_OK
            }
            Err(error) => error.into_c_int(),
        },
        iox2_service_type_e::LOCAL => match client
            .value
            .as_ref()
            .local
            .loan_custom_payload(number_of_elements)
        {
            Ok(request) => {
                let (request_struct_ptr, deleter) = init_request_struct_ptr(request_struct_ptr);
                (*request_struct_ptr).init(
                    client.service_type,
                    RequestMutUninitUnion::new_local(request),
                    deleter,
                );
                *request_handle_ptr = (*request_struct_ptr).as_handle();
                IOX2_OK
            }
            Err(error) => error.into_c_int(),
        },
    }
}

unsafe fn send_copy<S: Service>(
    client: &Client<S, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
    data_ptr: *const c_void,
    size_of_element: usize,
    number_of_elements: usize,
) -> Result<PendingResponse<S, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>, c_int> {
    let mut request = match client.loan_custom_payload(number_of_elements) {
        Ok(request) => request,
        Err(e) => return Err(e.into_c_int()),
    };

    let data_len = size_of_element * number_of_elements;
    if request.payload().len() < data_len {
        return Err(iox2_request_send_error_e::LOAN_ERROR_EXCEEDS_MAX_LOAN_SIZE as c_int);
    }

    let request_ptr = request.payload_mut().as_mut_ptr();
    core::ptr::copy_nonoverlapping(data_ptr, request_ptr.cast(), data_len);
    match request.assume_init().send() {
        Ok(pending_response) => Ok(pending_response),
        Err(e) => Err(e.into_c_int()),
    }
}

/// Sends a copy of the provided data via the client and provides a [`iox2_pending_response_h`]
/// to receive the corresponding responses.
///
/// # Arguments
///
/// * `client_handle` obtained by [`iox2_port_factory_client_builder_create`](crate::iox2_port_factory_client_builder_create)
/// * `pending_response_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_pending_response_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
/// * `pending_response_handle_ptr` - An uninitialized or dangling [`iox2_pending_response_h`] handle which will be initialized by this function call.
/// * `data_ptr` pointer to the payload that shall be transmitted
/// * `size_of_element` the size of the payload in bytes
/// * `number_of_elements` the number of elements in the payload
///
/// Return [`IOX2_OK`] on success, otherwise [`iox2_send_error_e`](crate::iox2_send_error_e).
///
/// # Safety
///
/// * `client_handle` is valid and non-null
/// * `data_ptr` non-null pointer to a valid position in memory
/// * `data_len` the size of the payload memory
/// * The `pending_response_handle_ptr` is pointing to a valid [`iox2_pending_response_h`].
#[no_mangle]
pub unsafe extern "C" fn iox2_client_send_copy(
    client_handle: iox2_client_h_ref,
    data_ptr: *const c_void,
    size_of_element: usize,
    number_of_elements: usize,
    pending_response_struct_ptr: *mut iox2_pending_response_t,
    pending_response_handle_ptr: *mut iox2_pending_response_h,
) -> c_int {
    client_handle.assert_non_null();
    debug_assert!(!data_ptr.is_null());
    debug_assert!(size_of_element != 0);

    let client = &mut *client_handle.as_type();

    let init_pending_response_struct_ptr =
        |pending_response_struct_ptr: *mut iox2_pending_response_t| {
            let mut pending_response_struct_ptr = pending_response_struct_ptr;
            fn no_op(_: *mut iox2_pending_response_t) {}
            let mut deleter: fn(*mut iox2_pending_response_t) = no_op;
            if pending_response_struct_ptr.is_null() {
                pending_response_struct_ptr = iox2_pending_response_t::alloc();
                deleter = iox2_pending_response_t::dealloc;
            }
            debug_assert!(!pending_response_struct_ptr.is_null());

            (pending_response_struct_ptr, deleter)
        };

    match client.service_type {
        iox2_service_type_e::IPC => match send_copy(
            &client.value.as_mut().ipc,
            data_ptr,
            size_of_element,
            number_of_elements,
        ) {
            Ok(pending_response) => {
                let (pending_response_struct_ptr, deleter) =
                    init_pending_response_struct_ptr(pending_response_struct_ptr);
                (*pending_response_struct_ptr).init(
                    client.service_type,
                    PendingResponseUnion::new_ipc(pending_response),
                    deleter,
                );
                *pending_response_handle_ptr = (*pending_response_struct_ptr).as_handle();
                IOX2_OK
            }
            Err(e) => e,
        },
        iox2_service_type_e::LOCAL => match send_copy(
            &client.value.as_mut().local,
            data_ptr,
            size_of_element,
            number_of_elements,
        ) {
            Ok(pending_response) => {
                let (pending_response_struct_ptr, deleter) =
                    init_pending_response_struct_ptr(pending_response_struct_ptr);
                (*pending_response_struct_ptr).init(
                    client.service_type,
                    PendingResponseUnion::new_local(pending_response),
                    deleter,
                );
                *pending_response_handle_ptr = (*pending_response_struct_ptr).as_handle();
                IOX2_OK
            }
            Err(e) => e,
        },
    }
}

/// This function needs to be called to destroy the client!
///
/// # Arguments
///
/// * `client_handle` - A valid [`iox2_client_h`]
///
/// # Safety
///
/// * The `client_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_client_t`] can be re-used with a call to
///   [`iox2_port_factory_client_builder_create`](crate::iox2_port_factory_client_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_client_drop(client_handle: iox2_client_h) {
    client_handle.assert_non_null();

    let client = &mut *client_handle.as_type();

    match client.service_type {
        iox2_service_type_e::IPC => {
            ManuallyDrop::drop(&mut client.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            ManuallyDrop::drop(&mut client.value.as_mut().local);
        }
    }
    (client.deleter)(client);
}
// END C API
