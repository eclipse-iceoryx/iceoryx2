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

use core::ffi::{c_int, c_void};
use core::mem::ManuallyDrop;

use crate::api::{IntoCInt, ResponseMutUninitUnion};
use crate::IOX2_OK;

use super::{
    c_size_t, iox2_request_header_h, iox2_request_header_t, iox2_response_mut_h,
    iox2_response_mut_t, iox2_send_error_e, iox2_service_type_e, AssertNonNullHandle, HandleToType,
    PayloadFfi, UserHeaderFfi,
};
use iceoryx2::active_request::ActiveRequest;
use iceoryx2::prelude::*;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

// BEGIN types definition
pub(super) union ActiveRequestUnion {
    ipc: ManuallyDrop<
        ActiveRequest<crate::IpcService, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
    >,
    local: ManuallyDrop<
        ActiveRequest<crate::LocalService, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
    >,
}

impl ActiveRequestUnion {
    pub(super) fn new_ipc(
        active_request: ActiveRequest<
            crate::IpcService,
            PayloadFfi,
            UserHeaderFfi,
            PayloadFfi,
            UserHeaderFfi,
        >,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(active_request),
        }
    }
    pub(super) fn new_local(
        active_request: ActiveRequest<
            crate::LocalService,
            PayloadFfi,
            UserHeaderFfi,
            PayloadFfi,
            UserHeaderFfi,
        >,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(active_request),
        }
    }
}

#[repr(C)]
#[repr(align(16))] // alignment of Option<ActiveRequestUnion>
pub struct iox2_active_request_storage_t {
    internal: [u8; 128], // magic number obtained with size_of::<Option<ActiveRequestUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(ActiveRequestUnion)]
pub struct iox2_active_request_t {
    service_type: iox2_service_type_e,
    value: iox2_active_request_storage_t,
    deleter: fn(*mut iox2_active_request_t),
}

impl iox2_active_request_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: ActiveRequestUnion,
        deleter: fn(*mut iox2_active_request_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_active_request_h_t;
/// The owning handle for `iox2_active_request_t`. Passing the handle to a function transfers the ownership.
pub type iox2_active_request_h = *mut iox2_active_request_h_t;
/// The non-owning handle for `iox2_active_request_t`. Passing the handle to a function does not transfer the ownership.
pub type iox2_active_request_h_ref = *const iox2_active_request_h;

impl AssertNonNullHandle for iox2_active_request_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_active_request_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_active_request_h {
    type Target = *mut iox2_active_request_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_active_request_h_ref {
    type Target = *mut iox2_active_request_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END types definition

// BEGIN C API

/// Returns true if the client wants to gracefully disconnect. This allows the Server to send its last
/// response and then drop the active request to signal the client that no more response will be sent.
///
/// # Arguments
///
/// * `handle` - Must be a valid [`iox2_active_request_h_ref`]
///   obtained by [`iox2_server_receive`](crate::iox2_server_receive).
///
/// # Safety
///
/// * `handle` must be a valid handle
#[no_mangle]
pub unsafe extern "C" fn iox2_active_request_has_disconnect_hint(
    handle: iox2_active_request_h_ref,
) -> bool {
    handle.assert_non_null();

    let active_request = &mut *handle.as_type();

    match active_request.service_type {
        iox2_service_type_e::IPC => active_request.value.as_mut().ipc.has_disconnect_hint(),
        iox2_service_type_e::LOCAL => active_request.value.as_mut().local.has_disconnect_hint(),
    }
}

/// Returns true if the corresponding pending response is still connected anc can receive responses.
///
/// # Arguments
///
/// * `handle` - Must be a valid [`iox2_active_request_h_ref`]
///   obtained by [`iox2_server_receive`](crate::iox2_server_receive).
///
/// # Safety
///
/// * `handle` must be a valid handle
#[no_mangle]
pub unsafe extern "C" fn iox2_active_request_is_connected(
    handle: iox2_active_request_h_ref,
) -> bool {
    handle.assert_non_null();

    let active_request = &mut *handle.as_type();

    match active_request.service_type {
        iox2_service_type_e::IPC => active_request.value.as_mut().ipc.is_connected(),
        iox2_service_type_e::LOCAL => active_request.value.as_mut().local.is_connected(),
    }
}

/// Acquires the requests header.
///
/// # Safety
///
/// * `handle` - Must be a valid [`iox2_active_request_h_ref`]
///   obtained by [`iox2_server_receive`](crate::iox2_server_receive).
/// * `header_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_request_header_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `header_handle_ptr` valid pointer to a [`iox2_request_header_h`].
#[no_mangle]
pub unsafe extern "C" fn iox2_active_request_header(
    handle: iox2_active_request_h_ref,
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

    let active_request = &mut *handle.as_type();

    let header = *match active_request.service_type {
        iox2_service_type_e::IPC => active_request.value.as_mut().ipc.header(),
        iox2_service_type_e::LOCAL => active_request.value.as_mut().local.header(),
    };

    (*storage_ptr).init(header, deleter);
    *header_handle_ptr = (*storage_ptr).as_handle();
}

/// Acquires the request user header.
///
/// # Safety
///
/// * `handle` - Must be a valid [`iox2_active_request_h_ref`]
///   obtained by [`iox2_server_receive`](crate::iox2_server_receive).
#[no_mangle]
pub unsafe extern "C" fn iox2_active_request_user_header(
    handle: iox2_active_request_h_ref,
    header_ptr: *mut *const c_void,
) {
    handle.assert_non_null();
    debug_assert!(!header_ptr.is_null());

    let active_request = &mut *handle.as_type();

    let header = match active_request.service_type {
        iox2_service_type_e::IPC => active_request.value.as_mut().ipc.user_header(),
        iox2_service_type_e::LOCAL => active_request.value.as_mut().local.user_header(),
    };

    *header_ptr = (header as *const UserHeaderFfi).cast();
}

/// Acquires the request payload.
///
/// # Safety
///
/// * `handle` - Must be a valid [`iox2_active_request_h_ref`]
///   obtained by [`iox2_server_receive`](crate::iox2_server_receive).
/// * `payload_ptr` a valid, non-null pointer pointing to a `*const c_void` pointer.
/// * `number_of_elements` (optional) either a null pointer or a valid pointer pointing to a [`c_size_t`].
#[no_mangle]
pub unsafe extern "C" fn iox2_active_request_payload(
    handle: iox2_active_request_h_ref,
    payload_ptr: *mut *const c_void,
    number_of_elements: *mut c_size_t,
) {
    handle.assert_non_null();
    debug_assert!(!payload_ptr.is_null());

    let active_request = &mut *handle.as_type();
    let number_of_elements_value;

    match active_request.service_type {
        iox2_service_type_e::IPC => {
            *payload_ptr = active_request.value.as_mut().ipc.payload().as_ptr().cast();
            number_of_elements_value = active_request
                .value
                .as_mut()
                .ipc
                .header()
                .number_of_elements();
        }
        iox2_service_type_e::LOCAL => {
            *payload_ptr = active_request
                .value
                .as_mut()
                .local
                .payload()
                .as_ptr()
                .cast();
            number_of_elements_value = active_request
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

/// Loans memory from the servers data segment.
///
/// # Arguments
///
/// * `active_request_handle` - Must be a valid [`iox2_active_request_h_ref`]
///   obtained by [`iox2_server_receive`](crate::iox2_server_receive).
/// * `response_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_response_mut_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
/// * `response_handle_ptr` - An uninitialized or dangling [`iox2_response_mut_h`] handle which will be initialized by this function call if a response is obtained, otherwise it will be set to NULL.
/// * `number_of_elements` - The number of elements to loan from the server's payload segment
///
/// Return [`IOX2_OK`] on success, otherwise [`iox2_loan_error_e`](crate::iox2_loan_error_e).
///
/// # Safety
///
/// * `active_request_handle` is valid and non-null
/// * The `response_handle_ptr` is pointing to a valid [`iox2_response_mut_h`].
#[no_mangle]
pub unsafe extern "C" fn iox2_active_request_loan_slice_uninit(
    active_request_handle: iox2_active_request_h_ref,
    response_struct_ptr: *mut iox2_response_mut_t,
    response_handle_ptr: *mut iox2_response_mut_h,
    number_of_elements: usize,
) -> c_int {
    active_request_handle.assert_non_null();
    debug_assert!(!response_handle_ptr.is_null());

    *response_handle_ptr = core::ptr::null_mut();

    let init_response_struct_ptr = |response_struct_ptr: *mut iox2_response_mut_t| {
        let mut response_struct_ptr = response_struct_ptr;
        fn no_op(_: *mut iox2_response_mut_t) {}
        let mut deleter: fn(*mut iox2_response_mut_t) = no_op;
        if response_struct_ptr.is_null() {
            response_struct_ptr = iox2_response_mut_t::alloc();
            deleter = iox2_response_mut_t::dealloc;
        }
        debug_assert!(!response_struct_ptr.is_null());

        (response_struct_ptr, deleter)
    };

    let active_request = &mut *active_request_handle.as_type();

    match active_request.service_type {
        iox2_service_type_e::IPC => match active_request
            .value
            .as_ref()
            .ipc
            .loan_custom_payload(number_of_elements)
        {
            Ok(response) => {
                let (response_struct_ptr, deleter) = init_response_struct_ptr(response_struct_ptr);
                (*response_struct_ptr).init(
                    active_request.service_type,
                    ResponseMutUninitUnion::new_ipc(response),
                    deleter,
                );
                *response_handle_ptr = (*response_struct_ptr).as_handle();
                IOX2_OK
            }
            Err(error) => error.into_c_int(),
        },
        iox2_service_type_e::LOCAL => match active_request
            .value
            .as_ref()
            .local
            .loan_custom_payload(number_of_elements)
        {
            Ok(response) => {
                let (response_struct_ptr, deleter) = init_response_struct_ptr(response_struct_ptr);
                (*response_struct_ptr).init(
                    active_request.service_type,
                    ResponseMutUninitUnion::new_local(response),
                    deleter,
                );
                *response_handle_ptr = (*response_struct_ptr).as_handle();
                IOX2_OK
            }
            Err(error) => error.into_c_int(),
        },
    }
}

unsafe fn send_copy<S: Service>(
    active_request: &ActiveRequest<S, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
    data_ptr: *const c_void,
    size_of_element: usize,
    number_of_elements: usize,
) -> c_int {
    let mut sample = match active_request.loan_custom_payload(number_of_elements) {
        Ok(sample) => sample,
        Err(e) => return e.into_c_int(),
    };

    let data_len = size_of_element * number_of_elements;
    if sample.payload().len() < data_len {
        return iox2_send_error_e::LOAN_ERROR_EXCEEDS_MAX_LOAN_SIZE as c_int;
    }

    let sample_ptr = sample.payload_mut().as_mut_ptr();
    core::ptr::copy_nonoverlapping(data_ptr, sample_ptr.cast(), data_len);
    match sample.assume_init().send() {
        Ok(()) => IOX2_OK,
        Err(e) => e.into_c_int(),
    }
}

/// Sends a copy of the provided data via the server.
///
/// # Arguments
///
/// * `active_request_handle` - Must be a valid [`iox2_active_request_h_ref`]
///   obtained by [`iox2_server_receive`](crate::iox2_server_receive).
/// * `data_ptr` pointer to the payload that shall be transmitted
/// * `size_of_element` the size of the payload in bytes
/// * `number_of_elements` the number of elements stored in data_ptr
///
/// Return [`IOX2_OK`] on success, otherwise [`iox2_send_error_e`].
///
/// # Safety
///
/// * `active_request_handle` is valid and non-null
/// * `data_ptr` non-null pointer to a valid position in memory
#[no_mangle]
pub unsafe extern "C" fn iox2_active_request_send_copy(
    active_request_handle: iox2_active_request_h_ref,
    data_ptr: *const c_void,
    size_of_element: usize,
    number_of_elements: usize,
) -> c_int {
    active_request_handle.assert_non_null();
    debug_assert!(!data_ptr.is_null());
    debug_assert!(size_of_element != 0);

    let active_request = &mut *active_request_handle.as_type();

    match active_request.service_type {
        iox2_service_type_e::IPC => send_copy(
            &active_request.value.as_mut().ipc,
            data_ptr,
            size_of_element,
            number_of_elements,
        ),
        iox2_service_type_e::LOCAL => send_copy(
            &active_request.value.as_mut().local,
            data_ptr,
            size_of_element,
            number_of_elements,
        ),
    }
}

/// This function needs to be called to destroy the active_request!
///
/// # Arguments
///
/// * `handle` - A valid [`iox2_active_request_h`]
///
/// # Safety
///
/// * The `handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_active_request_t`] can be re-used with a call to
///   [`iox2_server_receive`](crate::iox2_server_receive)!
#[no_mangle]
pub unsafe extern "C" fn iox2_active_request_drop(handle: iox2_active_request_h) {
    debug_assert!(!handle.is_null());

    let active_request = &mut *handle.as_type();

    match active_request.service_type {
        iox2_service_type_e::IPC => {
            ManuallyDrop::drop(&mut active_request.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            ManuallyDrop::drop(&mut active_request.value.as_mut().local);
        }
    }
    (active_request.deleter)(active_request);
}
// END C API
