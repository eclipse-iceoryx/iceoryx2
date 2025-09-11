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

// BEGIN types definition

use core::{ffi::c_int, ffi::c_void, mem::ManuallyDrop};

use iceoryx2::pending_response::PendingResponse;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use crate::{
    api::{IntoCInt, ResponseUnion},
    IOX2_OK,
};

use super::{
    c_size_t, iox2_request_header_h, iox2_request_header_t, iox2_response_h, iox2_response_t,
    iox2_service_type_e, AssertNonNullHandle, HandleToType, PayloadFfi, UserHeaderFfi,
};

pub(super) union PendingResponseUnion {
    ipc: ManuallyDrop<
        PendingResponse<crate::IpcService, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
    >,
    local: ManuallyDrop<
        PendingResponse<crate::LocalService, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
    >,
}

impl PendingResponseUnion {
    pub(super) fn new_ipc(
        pending_response: PendingResponse<
            crate::IpcService,
            PayloadFfi,
            UserHeaderFfi,
            PayloadFfi,
            UserHeaderFfi,
        >,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(pending_response),
        }
    }
    pub(super) fn new_local(
        pending_response: PendingResponse<
            crate::LocalService,
            PayloadFfi,
            UserHeaderFfi,
            PayloadFfi,
            UserHeaderFfi,
        >,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(pending_response),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<PendingResponseUnion>
pub struct iox2_pending_response_storage_t {
    internal: [u8; 88], // magic number obtained with size_of::<Option<PendingResponseUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(PendingResponseUnion)]
pub struct iox2_pending_response_t {
    service_type: iox2_service_type_e,
    value: iox2_pending_response_storage_t,
    deleter: fn(*mut iox2_pending_response_t),
}

impl iox2_pending_response_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: PendingResponseUnion,
        deleter: fn(*mut iox2_pending_response_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_pending_response_h_t;
/// The owning handle for `iox2_pending_response_t`. Passing the handle to a function transfers the ownership.
pub type iox2_pending_response_h = *mut iox2_pending_response_h_t;
/// The non-owning handle for `iox2_pending_response_t`. Passing the handle to a function does not transfer the ownership.
pub type iox2_pending_response_h_ref = *const iox2_pending_response_h;

impl AssertNonNullHandle for iox2_pending_response_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_pending_response_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_pending_response_h {
    type Target = *mut iox2_pending_response_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_pending_response_h_ref {
    type Target = *mut iox2_pending_response_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition

// BEGIN C API

/// Returns true if the corresponding active request is still connected and responses can send
/// further responses, otherwise false.
///
/// # Arguments
///
/// * `handle` - Must be a valid [`iox2_pending_response_h_ref`]
///   obtained by [`iox2_request_mut_send`](crate::iox2_request_mut_send).
///
/// # Safety
///
/// * `handle` must be valid a handle
#[no_mangle]
pub unsafe extern "C" fn iox2_pending_response_is_connected(
    handle: iox2_pending_response_h_ref,
) -> bool {
    handle.assert_non_null();

    let pending_response = &mut *handle.as_type();

    match pending_response.service_type {
        iox2_service_type_e::IPC => pending_response.value.as_ref().ipc.is_connected(),
        iox2_service_type_e::LOCAL => pending_response.value.as_ref().local.is_connected(),
    }
}

/// Marks the connection state that the Client wants to gracefully disconnect. When the
/// server reads it, it can send the last response and drop the
/// corresponding active request to terminate the connection ensuring that no response
/// is lost on the client side.
///
/// # Arguments
///
/// * `handle` - Must be a valid [`iox2_pending_response_h_ref`]
///   obtained by [`iox2_request_mut_send`](crate::iox2_request_mut_send).
///
/// # Safety
///
/// * `handle` must be valid a handle
#[no_mangle]
pub unsafe extern "C" fn iox2_pending_response_set_disconnect_hint(
    handle: iox2_pending_response_h_ref,
) {
    handle.assert_non_null();

    let pending_response = &mut *handle.as_type();

    match pending_response.service_type {
        iox2_service_type_e::IPC => pending_response.value.as_ref().ipc.set_disconnect_hint(),
        iox2_service_type_e::LOCAL => pending_response.value.as_ref().local.set_disconnect_hint(),
    }
}

/// Returns how many servers received the corresponding request initially.
///
/// # Arguments
///
/// * `handle` - Must be a valid [`iox2_pending_response_h_ref`]
///   obtained by [`iox2_request_mut_send`](crate::iox2_request_mut_send).
///
/// # Safety
///
/// * `handle` must be valid a handle
#[no_mangle]
pub unsafe extern "C" fn iox2_pending_response_number_of_server_connections(
    handle: iox2_pending_response_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let pending_response = &mut *handle.as_type();

    match pending_response.service_type {
        iox2_service_type_e::IPC => pending_response
            .value
            .as_ref()
            .ipc
            .number_of_server_connections(),
        iox2_service_type_e::LOCAL => pending_response
            .value
            .as_ref()
            .local
            .number_of_server_connections(),
    }
}

/// Returns true if there is a response in the buffer, otherwise false.
///
/// # Arguments
///
/// * `handle` - Must be a valid [`iox2_pending_response_h_ref`]
///   obtained by [`iox2_request_mut_send`](crate::iox2_request_mut_send).
///
/// # Safety
///
/// * `handle` must be valid a handle
#[no_mangle]
pub unsafe extern "C" fn iox2_pending_response_has_response(
    handle: iox2_pending_response_h_ref,
) -> bool {
    handle.assert_non_null();

    let pending_response = &mut *handle.as_type();

    match pending_response.service_type {
        iox2_service_type_e::IPC => pending_response.value.as_ref().ipc.has_response(),
        iox2_service_type_e::LOCAL => pending_response.value.as_ref().local.has_response(),
    }
}

/// Acquires the requests header.
///
/// # Safety
///
/// * `handle` - Must be a valid [`iox2_pending_response_h_ref`]
///   obtained by [`iox2_request_mut_send`](crate::iox2_request_mut_send).
/// * `header_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_request_header_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `header_handle_ptr` valid pointer to a [`iox2_request_header_h`].
#[no_mangle]
pub unsafe extern "C" fn iox2_pending_response_header(
    handle: iox2_pending_response_h_ref,
    header_struct_ptr: *mut iox2_request_header_t,
    header_handle_ptr: *mut iox2_request_header_h,
) {
    handle.assert_non_null();
    debug_assert!(!header_handle_ptr.is_null());

    fn no_op(_: *mut iox2_request_header_t) {}
    let mut deleter: fn(*mut iox2_request_header_t) = no_op;
    let mut storage_ptr = header_struct_ptr;
    if header_struct_ptr.is_null() {
        deleter = iox2_request_header_t::dealloc;
        storage_ptr = iox2_request_header_t::alloc();
    }
    debug_assert!(!storage_ptr.is_null());

    let sample = &mut *handle.as_type();

    let header = *match sample.service_type {
        iox2_service_type_e::IPC => sample.value.as_mut().ipc.header(),
        iox2_service_type_e::LOCAL => sample.value.as_mut().local.header(),
    };

    (*storage_ptr).init(header, deleter);
    *header_handle_ptr = (*storage_ptr).as_handle();
}

/// Acquires the requests user header.
///
/// # Safety
///
/// * `handle` - Must be a valid [`iox2_pending_response_h_ref`]
///   obtained by [`iox2_request_mut_send`](crate::iox2_request_mut_send).
/// * `header_ptr` a valid, non-null pointer pointing to a `*const c_void` pointer.
#[no_mangle]
pub unsafe extern "C" fn iox2_pending_response_user_header(
    handle: iox2_pending_response_h_ref,
    header_ptr: *mut *const c_void,
) {
    handle.assert_non_null();
    debug_assert!(!header_ptr.is_null());

    let pending_response = &mut *handle.as_type();

    let header = match pending_response.service_type {
        iox2_service_type_e::IPC => pending_response.value.as_mut().ipc.user_header(),
        iox2_service_type_e::LOCAL => pending_response.value.as_mut().local.user_header(),
    };

    *header_ptr = (header as *const UserHeaderFfi).cast();
}

/// Acquires the requests payload.
///
/// # Safety
///
/// * `handle` - Must be a valid [`iox2_pending_response_h_ref`]
///   obtained by [`iox2_request_mut_send`](crate::iox2_request_mut_send).
/// * `payload_ptr` a valid, non-null pointer pointing to a `*const c_void` pointer.
/// * `number_of_elements` (optional) either a null pointer or a valid pointer pointing to a [`c_size_t`].
#[no_mangle]
pub unsafe extern "C" fn iox2_pending_response_payload(
    handle: iox2_pending_response_h_ref,
    payload_ptr: *mut *const c_void,
    number_of_elements: *mut c_size_t,
) {
    handle.assert_non_null();
    debug_assert!(!payload_ptr.is_null());

    let pending_response = &mut *handle.as_type();
    let number_of_elements_value;

    match pending_response.service_type {
        iox2_service_type_e::IPC => {
            *payload_ptr = pending_response
                .value
                .as_mut()
                .ipc
                .payload()
                .as_ptr()
                .cast();
            number_of_elements_value = pending_response
                .value
                .as_mut()
                .ipc
                .header()
                .number_of_elements();
        }
        iox2_service_type_e::LOCAL => {
            *payload_ptr = pending_response
                .value
                .as_mut()
                .local
                .payload()
                .as_ptr()
                .cast();
            number_of_elements_value = pending_response
                .value
                .as_mut()
                .local
                .header()
                .number_of_elements();
        }
    };

    if !number_of_elements.is_null() {
        *number_of_elements = number_of_elements_value as c_size_t;
    }
}

/// Takes a response out of the buffer.
///
/// # Arguments
///
/// * `handle` - Must be a valid [`iox2_pending_response_h_ref`]
///   obtained by [`iox2_request_mut_send`](crate::iox2_request_mut_send).
/// * `response_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_response_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
/// * `response_handle_ptr` - An uninitialized or dangling [`iox2_response_h`] handle which will be initialized by this function call if a sample is obtained, otherwise it will be set to NULL.
///
/// Returns IOX2_OK on success, an [`iox2_receive_error_e`](crate::iox2_receive_error_e) otherwise.
/// Attention, an empty response buffer is not an error and even with IOX2_OK it is possible to get a NULL in `response_handle_ptr`.
///
/// # Safety
///
/// * The `handle` is still valid after the return of this function and can be use in another function call.
/// * The `response_handle_ptr` is pointing to a valid [`iox2_response_h`].
#[no_mangle]
pub unsafe extern "C" fn iox2_pending_response_receive(
    handle: iox2_pending_response_h_ref,
    response_struct_ptr: *mut iox2_response_t,
    response_handle_ptr: *mut iox2_response_h,
) -> c_int {
    handle.assert_non_null();
    debug_assert!(!response_handle_ptr.is_null());

    *response_handle_ptr = core::ptr::null_mut();

    let init_response_struct_ptr = |response_struct_ptr: *mut iox2_response_t| {
        let mut response_struct_ptr = response_struct_ptr;
        fn no_op(_: *mut iox2_response_t) {}
        let mut deleter: fn(*mut iox2_response_t) = no_op;
        if response_struct_ptr.is_null() {
            response_struct_ptr = iox2_response_t::alloc();
            deleter = iox2_response_t::dealloc;
        }
        debug_assert!(!response_struct_ptr.is_null());

        (response_struct_ptr, deleter)
    };

    let pending_response = &mut *handle.as_type();

    match pending_response.service_type {
        iox2_service_type_e::IPC => {
            match pending_response.value.as_ref().ipc.receive_custom_payload() {
                Ok(Some(response)) => {
                    let (response_struct_ptr, deleter) =
                        init_response_struct_ptr(response_struct_ptr);
                    (*response_struct_ptr).init(
                        pending_response.service_type,
                        ResponseUnion::new_ipc(response),
                        deleter,
                    );
                    *response_handle_ptr = (*response_struct_ptr).as_handle();
                }
                Ok(None) => (),
                Err(error) => return error.into_c_int(),
            }
        }
        iox2_service_type_e::LOCAL => {
            match pending_response
                .value
                .as_ref()
                .local
                .receive_custom_payload()
            {
                Ok(Some(response)) => {
                    let (response_struct_ptr, deleter) =
                        init_response_struct_ptr(response_struct_ptr);
                    (*response_struct_ptr).init(
                        pending_response.service_type,
                        ResponseUnion::new_local(response),
                        deleter,
                    );
                    *response_handle_ptr = (*response_struct_ptr).as_handle();
                }
                Ok(None) => (),
                Err(error) => return error.into_c_int(),
            }
        }
    }

    IOX2_OK
}

/// This function needs to be called to destroy the pending response!
///
/// # Arguments
///
/// * `handle` - A valid [`iox2_pending_response_h`]
///
/// # Safety
///
/// * The `handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_pending_response_t`] can be re-used with a call to
///   [`iox2_request_mut_send`](crate::iox2_request_mut_send)!
#[no_mangle]
pub unsafe extern "C" fn iox2_pending_response_drop(handle: iox2_pending_response_h) {
    debug_assert!(!handle.is_null());

    let pending_response = &mut *handle.as_type();

    match pending_response.service_type {
        iox2_service_type_e::IPC => {
            ManuallyDrop::drop(&mut pending_response.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            ManuallyDrop::drop(&mut pending_response.value.as_mut().local);
        }
    }
    (pending_response.deleter)(pending_response);
}
// END C API
