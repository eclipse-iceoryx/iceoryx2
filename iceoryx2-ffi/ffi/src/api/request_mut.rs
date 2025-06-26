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

use crate::api::PendingResponseUnion;
use crate::api::{
    c_size_t, iox2_service_type_e, AssertNonNullHandle, HandleToType, IntoCInt, UserHeaderFfi,
    IOX2_OK,
};

use iceoryx2::port::client::RequestSendError;
use iceoryx2::port::LoanError;
use iceoryx2::port::SendError;
use iceoryx2::request_mut_uninit::RequestMutUninit;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_bb_elementary_traits::AsCStr;
use iceoryx2_ffi_macros::{iceoryx2_ffi, CStrRepr};

use core::ffi::{c_char, c_int, c_void};
use core::mem::ManuallyDrop;

use super::iox2_pending_response_h;
use super::iox2_pending_response_t;
use super::{iox2_request_header_h, iox2_request_header_t, PayloadFfi, UninitPayloadFfi};

// BEGIN types definition
#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_request_send_error_e {
    CONNECTION_BROKEN_SINCE_SENDER_NO_LONGER_EXISTS = IOX2_OK as isize + 1,
    CONNECTION_CORRUPTED,
    LOAN_ERROR_OUT_OF_MEMORY,
    LOAN_ERROR_EXCEEDS_MAX_LOANS,
    LOAN_ERROR_EXCEEDS_MAX_LOAN_SIZE,
    LOAN_ERROR_INTERNAL_FAILURE,
    CONNECTION_ERROR,
    EXCEEDS_MAX_ACTIVE_REQUESTS,
}

impl IntoCInt for RequestSendError {
    fn into_c_int(self) -> c_int {
        (match self {
            RequestSendError::SendError(SendError::ConnectionBrokenSinceSenderNoLongerExists) => {
                iox2_request_send_error_e::CONNECTION_BROKEN_SINCE_SENDER_NO_LONGER_EXISTS
            }
            RequestSendError::SendError(SendError::ConnectionCorrupted) => {
                iox2_request_send_error_e::CONNECTION_CORRUPTED
            }
            RequestSendError::SendError(SendError::LoanError(LoanError::OutOfMemory)) => {
                iox2_request_send_error_e::LOAN_ERROR_OUT_OF_MEMORY
            }
            RequestSendError::SendError(SendError::LoanError(LoanError::ExceedsMaxLoans)) => {
                iox2_request_send_error_e::LOAN_ERROR_EXCEEDS_MAX_LOANS
            }
            RequestSendError::SendError(SendError::LoanError(LoanError::ExceedsMaxLoanSize)) => {
                iox2_request_send_error_e::LOAN_ERROR_EXCEEDS_MAX_LOAN_SIZE
            }
            RequestSendError::SendError(SendError::LoanError(LoanError::InternalFailure)) => {
                iox2_request_send_error_e::LOAN_ERROR_INTERNAL_FAILURE
            }
            RequestSendError::SendError(SendError::ConnectionError(_)) => {
                iox2_request_send_error_e::CONNECTION_ERROR
            }
            RequestSendError::ExceedsMaxActiveRequests => {
                iox2_request_send_error_e::EXCEEDS_MAX_ACTIVE_REQUESTS
            }
        }) as c_int
    }
}

pub(super) union RequestMutUninitUnion {
    ipc: ManuallyDrop<
        RequestMutUninit<
            crate::IpcService,
            UninitPayloadFfi,
            UserHeaderFfi,
            PayloadFfi,
            UserHeaderFfi,
        >,
    >,
    local: ManuallyDrop<
        RequestMutUninit<
            crate::LocalService,
            UninitPayloadFfi,
            UserHeaderFfi,
            PayloadFfi,
            UserHeaderFfi,
        >,
    >,
}

impl RequestMutUninitUnion {
    pub(super) fn new_ipc(
        sample: RequestMutUninit<
            crate::IpcService,
            UninitPayloadFfi,
            UserHeaderFfi,
            PayloadFfi,
            UserHeaderFfi,
        >,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(sample),
        }
    }
    pub(super) fn new_local(
        sample: RequestMutUninit<
            crate::LocalService,
            UninitPayloadFfi,
            UserHeaderFfi,
            PayloadFfi,
            UserHeaderFfi,
        >,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(sample),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<RequestMutUninitUnion>
pub struct iox2_request_mut_storage_t {
    internal: [u8; 80], // magic number obtained with size_of::<Option<RequestMutUninitUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(RequestMutUninitUnion)]
pub struct iox2_request_mut_t {
    service_type: iox2_service_type_e,
    value: iox2_request_mut_storage_t,
    deleter: fn(*mut iox2_request_mut_t),
}

impl iox2_request_mut_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: RequestMutUninitUnion,
        deleter: fn(*mut iox2_request_mut_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_request_mut_h_t;
/// The owning handle for `iox2_request_mut_t`. Passing the handle to a function transfers the ownership.
pub type iox2_request_mut_h = *mut iox2_request_mut_h_t;
/// The non-owning handle for `iox2_request_mut_t`. Passing the handle to a function does not transfer the ownership.
pub type iox2_request_mut_h_ref = *const iox2_request_mut_h;

impl AssertNonNullHandle for iox2_request_mut_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_request_mut_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_request_mut_h {
    type Target = *mut iox2_request_mut_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_request_mut_h_ref {
    type Target = *mut iox2_request_mut_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition

// BEGIN C API

/// Returns a string literal describing the provided [`iox2_request_send_error_e`].
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
/// The returned pointer must not be modified or freed and is valid as long as the program runs.
#[no_mangle]
pub unsafe extern "C" fn iox2_request_send_error_string(
    error: iox2_request_send_error_e,
) -> *const c_char {
    error.as_const_cstr().as_ptr() as *const c_char
}

/// cbindgen:ignore
/// Internal API - do not use
/// # Safety
///
/// * `source_struct_ptr` must not be `null` and the struct it is pointing to must be initialized and valid, i.e. not moved or dropped.
/// * `dest_struct_ptr` must not be `null` and the struct it is pointing to must not contain valid data, i.e. initialized. It can be moved or dropped, though.
/// * `dest_handle_ptr` must not be `null`
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn iox2_request_mut_move(
    source_struct_ptr: *mut iox2_request_mut_t,
    dest_struct_ptr: *mut iox2_request_mut_t,
    dest_handle_ptr: *mut iox2_request_mut_h,
) {
    debug_assert!(!source_struct_ptr.is_null());
    debug_assert!(!dest_struct_ptr.is_null());
    debug_assert!(!dest_handle_ptr.is_null());

    let source = &mut *source_struct_ptr;
    let dest = &mut *dest_struct_ptr;

    dest.service_type = source.service_type;
    dest.value.init(
        source
            .value
            .as_option_mut()
            .take()
            .expect("Source must have a valid request"),
    );
    dest.deleter = source.deleter;

    *dest_handle_ptr = (*dest_struct_ptr).as_handle();
}

/// Acquires the requests user header.
///
/// # Safety
///
/// * `handle` obtained by [`iox2_client_loan_slice_uninit()`](crate::iox2_client_loan_slice_uninit())
/// * `header_ptr` a valid, non-null pointer pointing to a `*const c_void` pointer.
#[no_mangle]
pub unsafe extern "C" fn iox2_request_mut_user_header(
    handle: iox2_request_mut_h_ref,
    header_ptr: *mut *const c_void,
) {
    handle.assert_non_null();
    debug_assert!(!header_ptr.is_null());

    let request = &mut *handle.as_type();

    let header = match request.service_type {
        iox2_service_type_e::IPC => request.value.as_mut().ipc.user_header(),
        iox2_service_type_e::LOCAL => request.value.as_mut().local.user_header(),
    };

    *header_ptr = (header as *const UserHeaderFfi).cast();
}

/// Acquires the requests header.
///
/// # Safety
///
/// * `handle` obtained by [`iox2_client_loan_slice_uninit()`](crate::iox2_client_loan_slice_uninit())
/// * `header_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_request_header_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `header_handle_ptr` valid pointer to a [`iox2_request_header_h`].
#[no_mangle]
pub unsafe extern "C" fn iox2_request_mut_header(
    handle: iox2_request_mut_h_ref,
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

/// Acquires the requests mutable user header.
///
/// # Safety
///
/// * `handle` obtained by [`iox2_client_loan_slice_uninit()`](crate::iox2_client_loan_slice_uninit())
/// * `header_ptr` a valid, non-null pointer pointing to a [`*const c_void`] pointer.
#[no_mangle]
pub unsafe extern "C" fn iox2_request_mut_user_header_mut(
    handle: iox2_request_mut_h_ref,
    header_ptr: *mut *mut c_void,
) {
    handle.assert_non_null();
    debug_assert!(!header_ptr.is_null());

    let request = &mut *handle.as_type();

    let header = match request.service_type {
        iox2_service_type_e::IPC => request.value.as_mut().ipc.user_header_mut(),
        iox2_service_type_e::LOCAL => request.value.as_mut().local.user_header_mut(),
    };

    *header_ptr = (header as *mut UserHeaderFfi).cast();
}

/// Acquires the requests mutable payload.
///
/// # Safety
///
/// * `handle` obtained by [`iox2_client_loan_slice_uninit()`](crate::iox2_client_loan_slice_uninit())
/// * `payload_ptr` a valid, non-null pointer pointing to a `*mut c_void` pointer.
/// * `number_of_elements` (optional) either a null pointer or a valid pointer pointing to a [`c_size_t`].
#[no_mangle]
pub unsafe extern "C" fn iox2_request_mut_payload_mut(
    handle: iox2_request_mut_h_ref,
    payload_ptr: *mut *mut c_void,
    number_of_elements: *mut c_size_t,
) {
    handle.assert_non_null();
    debug_assert!(!payload_ptr.is_null());

    let request = &mut *handle.as_type();
    let payload = request.value.as_mut().ipc.payload_mut();

    match request.service_type {
        iox2_service_type_e::IPC => {
            *payload_ptr = payload.as_mut_ptr().cast();
        }
        iox2_service_type_e::LOCAL => {
            *payload_ptr = payload.as_mut_ptr().cast();
        }
    };

    if !number_of_elements.is_null() {
        *number_of_elements =
            request.value.as_mut().local.header().number_of_elements() as c_size_t;
    }
}

/// Acquires the request payload.
///
/// # Safety
///
/// * `handle` obtained by [`iox2_client_loan_slice_uninit()`](crate::iox2_client_loan_slice_uninit())
/// * `payload_ptr` a valid, non-null pointer pointing to a [`*const c_void`] pointer.
/// * `number_of_elements` (optional) either a null pointer or a valid pointer pointing to a [`c_size_t`].
#[no_mangle]
pub unsafe extern "C" fn iox2_request_mut_payload(
    handle: iox2_request_mut_h_ref,
    payload_ptr: *mut *const c_void,
    number_of_elements: *mut c_size_t,
) {
    handle.assert_non_null();
    debug_assert!(!payload_ptr.is_null());

    let request = &mut *handle.as_type();
    let payload = request.value.as_mut().ipc.payload_mut();

    match request.service_type {
        iox2_service_type_e::IPC => {
            *payload_ptr = payload.as_mut_ptr().cast();
        }
        iox2_service_type_e::LOCAL => {
            *payload_ptr = payload.as_mut_ptr().cast();
        }
    };

    if !number_of_elements.is_null() {
        *number_of_elements =
            request.value.as_mut().local.header().number_of_elements() as c_size_t;
    }
}

/// Takes the ownership of the request and sends it
///
/// # Safety
///
/// * `handle` obtained by [`iox2_client_loan_slice_uninit()`](crate::iox2_client_loan_slice_uninit())
/// * The `pending_response_handle_ptr` is pointing to a valid [`iox2_pending_response_h`].
///
#[no_mangle]
pub unsafe extern "C" fn iox2_request_mut_send(
    handle: iox2_request_mut_h,
    pending_response_struct_ptr: *mut iox2_pending_response_t,
    pending_response_handle_ptr: *mut iox2_pending_response_h,
) -> c_int {
    debug_assert!(!handle.is_null());
    debug_assert!(!pending_response_handle_ptr.is_null());

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

    let request_struct = &mut *handle.as_type();
    let service_type = request_struct.service_type;

    let request = request_struct
        .value
        .as_option_mut()
        .take()
        .unwrap_or_else(|| panic!("Trying to send an already sent request!"));
    (request_struct.deleter)(request_struct);

    match service_type {
        iox2_service_type_e::IPC => {
            let request = ManuallyDrop::into_inner(request.ipc);
            match request.assume_init().send() {
                Ok(pending_response) => {
                    let (pending_response_struct_ptr, deleter) =
                        init_pending_response_struct_ptr(pending_response_struct_ptr);
                    (*pending_response_struct_ptr).init(
                        service_type,
                        PendingResponseUnion::new_ipc(pending_response),
                        deleter,
                    );
                    *pending_response_handle_ptr = (*pending_response_struct_ptr).as_handle();
                    IOX2_OK
                }
                Err(e) => e.into_c_int(),
            }
        }
        iox2_service_type_e::LOCAL => {
            let request = ManuallyDrop::into_inner(request.local);
            match request.assume_init().send() {
                Ok(pending_response) => {
                    let (pending_response_struct_ptr, deleter) =
                        init_pending_response_struct_ptr(pending_response_struct_ptr);
                    (*pending_response_struct_ptr).init(
                        service_type,
                        PendingResponseUnion::new_local(pending_response),
                        deleter,
                    );
                    *pending_response_handle_ptr = (*pending_response_struct_ptr).as_handle();
                    IOX2_OK
                }
                Err(e) => e.into_c_int(),
            }
        }
    }
}

/// This function needs to be called to destroy the request!
///
/// # Arguments
///
/// * `handle` - A valid [`iox2_request_mut_h`]
///
/// # Safety
///
/// * The `handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_request_mut_t`] can be re-used with a call to
///   [`iox2_client_loan_slice_uninit`](crate::iox2_client_loan_slice_uninit)!
#[no_mangle]
pub unsafe extern "C" fn iox2_request_mut_drop(handle: iox2_request_mut_h) {
    debug_assert!(!handle.is_null());

    let request = &mut *handle.as_type();

    match request.service_type {
        iox2_service_type_e::IPC => {
            ManuallyDrop::drop(&mut request.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            ManuallyDrop::drop(&mut request.value.as_mut().local);
        }
    }
    (request.deleter)(request);
}

// END C API
