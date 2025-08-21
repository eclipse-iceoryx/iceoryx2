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

use core::{
    alloc::Layout,
    ffi::{c_char, c_int},
    mem::ManuallyDrop,
    slice,
};

use iceoryx2::service::port_factory::request_response::PortFactory;
use iceoryx2::service::static_config::message_type_details::TypeDetail;
use iceoryx2::service::{
    builder::request_response::{
        Builder, RequestResponseCreateError, RequestResponseOpenOrCreateError,
    },
    static_config::message_type_details::TypeNameString,
};
use iceoryx2::{prelude::*, service::builder::request_response::RequestResponseOpenError};
use iceoryx2_bb_elementary_traits::AsCStr;
use iceoryx2_ffi_macros::CStrRepr;

use crate::{
    api::{
        AssertNonNullHandle, HandleToType, PortFactoryRequestResponseUnion, ServiceBuilderUnion,
    },
    iox2_service_type_e, iox2_type_detail_error_e, IOX2_OK,
};

use super::{
    c_size_t, iox2_attribute_specifier_h_ref, iox2_attribute_verifier_h_ref,
    iox2_port_factory_request_response_h, iox2_port_factory_request_response_t,
    iox2_service_builder_request_response_h, iox2_service_builder_request_response_h_ref,
    iox2_type_variant_e, IntoCInt, PayloadFfi, UserHeaderFfi,
};

// BEGIN types definition
#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_request_response_open_or_create_error_e {
    #[CStr = "does not exist"]
    O_DOES_NOT_EXIST = IOX2_OK as isize + 1,
    #[CStr = "does not support requested amount of client request loans"]
    O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_CLIENT_REQUEST_LOANS,
    #[CStr = "does not support requested amount of active requests per client"]
    O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_ACTIVE_REQUESTS_PER_CLIENT,
    #[CStr = "does not support requested response buffer size"]
    O_DOES_NOT_SUPPORT_REQUESTED_RESPONSE_BUFFER_SIZE,
    #[CStr = "does not support requested amount of servers"]
    O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_SERVERS,
    #[CStr = "does not support requested amount of clients"]
    O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_CLIENTS,
    #[CStr = "does not support requested amount of nodes"]
    O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NODES,
    #[CStr = "does not support requested amount of borrowed responses per pending response"]
    O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_BORROWED_RESPONSES_PER_PENDING_RESPONSE,
    #[CStr = "exceeds max number of nodes"]
    O_EXCEEDS_MAX_NUMBER_OF_NODES,
    #[CStr = "hangs in creation"]
    O_HANGS_IN_CREATION,
    #[CStr = "incompatible request type"]
    O_INCOMPATIBLE_REQUEST_TYPE,
    #[CStr = "incompatible response type"]
    O_INCOMPATIBLE_RESPONSE_TYPE,
    #[CStr = "incompatible attributes"]
    O_INCOMPATIBLE_ATTRIBUTES,
    #[CStr = "incompatible messaging pattern"]
    O_INCOMPATIBLE_MESSAGING_PATTERN,
    #[CStr = "incompatible overflow behavior for requests"]
    O_INCOMPATIBLE_OVERFLOW_BEHAVIOR_FOR_REQUESTS,
    #[CStr = "incompatible overflow behavior for responses"]
    O_INCOMPATIBLE_OVERFLOW_BEHAVIOR_FOR_RESPONSES,
    #[CStr = "incompatible behavior for fire and forget requests"]
    O_INCOMPATIBLE_BEHAVIOR_FOR_FIRE_AND_FORGET_REQUESTS,
    #[CStr = "insufficient permissions"]
    O_INSUFFICIENT_PERMISSIONS,
    #[CStr = "internal failure"]
    O_INTERNAL_FAILURE,
    #[CStr = "is marked for destruction"]
    O_IS_MARKED_FOR_DESTRUCTION,
    #[CStr = "service in corrupted state"]
    O_SERVICE_IN_CORRUPTED_STATE,
    #[CStr = "already exists"]
    C_ALREADY_EXISTS,
    #[CStr = "internal failure"]
    C_INTERNAL_FAILURE,
    #[CStr = "is being created by another instance"]
    C_IS_BEING_CREATED_BY_ANOTHER_INSTANCE,
    #[CStr = "insufficient permissions"]
    C_INSUFFICIENT_PERMISSIONS,
    #[CStr = "hangs in creation"]
    C_HANGS_IN_CREATION,
    #[CStr = "service in corrupted state"]
    C_SERVICE_IN_CORRUPTED_STATE,
    #[CStr = "system in flux"]
    SYSTEM_IN_FLUX,
}

impl IntoCInt for RequestResponseOpenError {
    fn into_c_int(self) -> c_int {
        (match self {
            RequestResponseOpenError::DoesNotExist => iox2_request_response_open_or_create_error_e::O_DOES_NOT_EXIST,
            RequestResponseOpenError::DoesNotSupportRequestedAmountOfClientRequestLoans => iox2_request_response_open_or_create_error_e::O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_CLIENT_REQUEST_LOANS,
            RequestResponseOpenError::DoesNotSupportRequestedAmountOfActiveRequestsPerClient => iox2_request_response_open_or_create_error_e::O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_ACTIVE_REQUESTS_PER_CLIENT,
            RequestResponseOpenError::DoesNotSupportRequestedResponseBufferSize => iox2_request_response_open_or_create_error_e::O_DOES_NOT_SUPPORT_REQUESTED_RESPONSE_BUFFER_SIZE,
            RequestResponseOpenError::DoesNotSupportRequestedAmountOfServers => iox2_request_response_open_or_create_error_e::O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_SERVERS,
            RequestResponseOpenError::DoesNotSupportRequestedAmountOfClients => iox2_request_response_open_or_create_error_e::O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_CLIENTS,
            RequestResponseOpenError::DoesNotSupportRequestedAmountOfNodes => iox2_request_response_open_or_create_error_e::O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NODES,
            RequestResponseOpenError::DoesNotSupportRequestedAmountOfBorrowedResponsesPerPendingResponse => iox2_request_response_open_or_create_error_e::O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_BORROWED_RESPONSES_PER_PENDING_RESPONSE,
            RequestResponseOpenError::ExceedsMaxNumberOfNodes => iox2_request_response_open_or_create_error_e::O_EXCEEDS_MAX_NUMBER_OF_NODES,
            RequestResponseOpenError::HangsInCreation => iox2_request_response_open_or_create_error_e::O_HANGS_IN_CREATION,
            RequestResponseOpenError::IncompatibleRequestType => iox2_request_response_open_or_create_error_e::O_INCOMPATIBLE_REQUEST_TYPE,
            RequestResponseOpenError::IncompatibleResponseType => iox2_request_response_open_or_create_error_e::O_INCOMPATIBLE_RESPONSE_TYPE,
            RequestResponseOpenError::IncompatibleAttributes => iox2_request_response_open_or_create_error_e::O_INCOMPATIBLE_ATTRIBUTES,
            RequestResponseOpenError::IncompatibleMessagingPattern => iox2_request_response_open_or_create_error_e::O_INCOMPATIBLE_MESSAGING_PATTERN,
            RequestResponseOpenError::IncompatibleOverflowBehaviorForRequests => iox2_request_response_open_or_create_error_e::O_INCOMPATIBLE_OVERFLOW_BEHAVIOR_FOR_REQUESTS,
            RequestResponseOpenError::IncompatibleOverflowBehaviorForResponses => iox2_request_response_open_or_create_error_e::O_INCOMPATIBLE_OVERFLOW_BEHAVIOR_FOR_RESPONSES,
            RequestResponseOpenError::IncompatibleBehaviorForFireAndForgetRequests => iox2_request_response_open_or_create_error_e::O_INCOMPATIBLE_BEHAVIOR_FOR_FIRE_AND_FORGET_REQUESTS,
            RequestResponseOpenError::InsufficientPermissions => iox2_request_response_open_or_create_error_e::O_INSUFFICIENT_PERMISSIONS,
            RequestResponseOpenError::InternalFailure => iox2_request_response_open_or_create_error_e::O_INTERNAL_FAILURE,
            RequestResponseOpenError::IsMarkedForDestruction => iox2_request_response_open_or_create_error_e::O_IS_MARKED_FOR_DESTRUCTION,
            RequestResponseOpenError::ServiceInCorruptedState => iox2_request_response_open_or_create_error_e::O_SERVICE_IN_CORRUPTED_STATE,
        }) as c_int
    }
}

impl IntoCInt for RequestResponseCreateError {
    fn into_c_int(self) -> c_int {
        (match self {
            RequestResponseCreateError::AlreadyExists => {
                iox2_request_response_open_or_create_error_e::C_ALREADY_EXISTS
            }
            RequestResponseCreateError::InternalFailure => {
                iox2_request_response_open_or_create_error_e::C_INTERNAL_FAILURE
            }
            RequestResponseCreateError::IsBeingCreatedByAnotherInstance => {
                iox2_request_response_open_or_create_error_e::C_IS_BEING_CREATED_BY_ANOTHER_INSTANCE
            }
            RequestResponseCreateError::InsufficientPermissions => {
                iox2_request_response_open_or_create_error_e::C_INSUFFICIENT_PERMISSIONS
            }
            RequestResponseCreateError::HangsInCreation => {
                iox2_request_response_open_or_create_error_e::C_HANGS_IN_CREATION
            }
            RequestResponseCreateError::ServiceInCorruptedState => {
                iox2_request_response_open_or_create_error_e::C_SERVICE_IN_CORRUPTED_STATE
            }
        }) as c_int
    }
}

impl IntoCInt for RequestResponseOpenOrCreateError {
    fn into_c_int(self) -> c_int {
        match self {
            RequestResponseOpenOrCreateError::RequestResponseOpenError(e) => e.into_c_int(),
            RequestResponseOpenOrCreateError::RequestResponseCreateError(e) => e.into_c_int(),
            RequestResponseOpenOrCreateError::SystemInFlux => {
                iox2_request_response_open_or_create_error_e::SYSTEM_IN_FLUX as c_int
            }
        }
    }
}

// END types definition

/// Returns a string literal describing the provided [`iox2_request_response_open_or_create_error_e`].
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
pub unsafe extern "C" fn iox2_request_response_open_or_create_error_string(
    error: iox2_request_response_open_or_create_error_e,
) -> *const c_char {
    error.as_const_cstr().as_ptr() as *const c_char
}

pub(crate) unsafe fn create_type_details(
    type_variant: iox2_type_variant_e,
    type_name_str: *const c_char,
    type_name_len: c_size_t,
    size: c_size_t,
    alignment: c_size_t,
) -> Result<TypeDetail, c_int> {
    debug_assert!(!type_name_str.is_null());

    let type_name = slice::from_raw_parts(type_name_str as _, type_name_len as _);

    let type_name = if let Ok(type_name) = core::str::from_utf8(type_name) {
        type_name.to_string()
    } else {
        return Err(iox2_type_detail_error_e::INVALID_TYPE_NAME as c_int);
    };

    let type_name = if let Ok(type_name) = TypeNameString::try_from(type_name.as_str()) {
        type_name
    } else {
        return Err(iox2_type_detail_error_e::INVALID_TYPE_NAME as c_int);
    };

    match Layout::from_size_align(size, alignment) {
        Ok(_) => (),
        Err(_) => return Err(iox2_type_detail_error_e::INVALID_SIZE_OR_ALIGNMENT_VALUE as c_int),
    }

    let mut type_detail = TypeDetail::new::<()>(type_variant.into());
    iceoryx2::testing::type_detail_set_name(&mut type_detail, type_name);
    iceoryx2::testing::type_detail_set_size(&mut type_detail, size);
    iceoryx2::testing::type_detail_set_alignment(&mut type_detail, alignment);

    Ok(type_detail)
}

/// Sets the request header type details for the builder
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_request_response_h_ref`]
///   obtained by [`iox2_service_builder_request_response`](crate::iox2_service_builder_request_response).
/// * `type_variant` - The [`iox2_type_variant_e`] for the payload
/// * `type_name_str` - string for the type name.
/// * `type_name_len` - The length of the type name string, not including a null
/// * `size` - The size of the payload
/// * `alignment` - The alignment of the payload
///
/// Returns IOX2_OK on success, an [`iox2_type_detail_error_e`] otherwise.
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
/// * `type_name_str` must be a valid pointer to an utf8 string
/// * `size` and `alignment` must satisfy the Rust `Layout` type requirements
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_request_response_set_request_header_type_details(
    service_builder_handle: iox2_service_builder_request_response_h_ref,
    type_variant: iox2_type_variant_e,
    type_name_str: *const c_char,
    type_name_len: c_size_t,
    size: c_size_t,
    alignment: c_size_t,
) -> c_int {
    service_builder_handle.assert_non_null();
    let value =
        match create_type_details(type_variant, type_name_str, type_name_len, size, alignment) {
            Ok(v) => v,
            Err(e) => return e,
        };

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_request_response(
                service_builder.__internal_set_request_header_type_details(&value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_local_request_response(
                service_builder.__internal_set_request_header_type_details(&value),
            ));
        }
    }

    IOX2_OK
}

/// Sets the response header type details for the builder
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_request_response_h_ref`]
///   obtained by [`iox2_service_builder_request_response`](crate::iox2_service_builder_request_response).
/// * `type_variant` - The [`iox2_type_variant_e`] for the payload
/// * `type_name_str` - string for the type name.
/// * `type_name_len` - The length of the type name string, not including a null
/// * `size` - The size of the payload
/// * `alignment` - The alignment of the payload
///
/// Returns IOX2_OK on success, an [`iox2_type_detail_error_e`] otherwise.
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
/// * `type_name_str` must be a valid pointer to an utf8 string
/// * `size` and `alignment` must satisfy the Rust `Layout` type requirements
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_request_response_set_response_header_type_details(
    service_builder_handle: iox2_service_builder_request_response_h_ref,
    type_variant: iox2_type_variant_e,
    type_name_str: *const c_char,
    type_name_len: c_size_t,
    size: c_size_t,
    alignment: c_size_t,
) -> c_int {
    service_builder_handle.assert_non_null();
    let value =
        match create_type_details(type_variant, type_name_str, type_name_len, size, alignment) {
            Ok(v) => v,
            Err(e) => return e,
        };

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_request_response(
                service_builder.__internal_set_response_header_type_details(&value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_local_request_response(
                service_builder.__internal_set_response_header_type_details(&value),
            ));
        }
    }

    IOX2_OK
}

/// Sets the request payload type details for the builder
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_request_response_h_ref`]
///   obtained by [`iox2_service_builder_request_response`](crate::iox2_service_builder_request_response).
/// * `type_variant` - The [`iox2_type_variant_e`] for the payload
/// * `type_name_str` - Must string for the type name.
/// * `type_name_len` - The length of the type name string, not including a null
/// * `size` - The size of the payload
/// * `alignment` - The alignment of the payload
///
/// Returns IOX2_OK on success, an [`iox2_type_detail_error_e`] otherwise.
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
/// * `type_name_str` must be a valid pointer to an utf8 string
/// * `size` and `alignment` must satisfy the Rust `Layout` type requirements
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_request_response_set_request_payload_type_details(
    service_builder_handle: iox2_service_builder_request_response_h_ref,
    type_variant: iox2_type_variant_e,
    type_name_str: *const c_char,
    type_name_len: c_size_t,
    size: c_size_t,
    alignment: c_size_t,
) -> c_int {
    service_builder_handle.assert_non_null();

    let value =
        match create_type_details(type_variant, type_name_str, type_name_len, size, alignment) {
            Ok(v) => v,
            Err(e) => return e,
        };

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_request_response(
                service_builder.__internal_set_request_payload_type_details(&value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_local_request_response(
                service_builder.__internal_set_request_payload_type_details(&value),
            ));
        }
    }

    IOX2_OK
}

/// Sets the response payload type details for the builder
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_request_response_h_ref`]
///   obtained by [`iox2_service_builder_request_response`](crate::iox2_service_builder_request_response).
/// * `type_variant` - The [`iox2_type_variant_e`] for the payload
/// * `type_name_str` - Must string for the type name.
/// * `type_name_len` - The length of the type name string, not including a null
/// * `size` - The size of the payload
/// * `alignment` - The alignment of the payload
///
/// Returns IOX2_OK on success, an [`iox2_type_detail_error_e`] otherwise.
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
/// * `type_name_str` must be a valid pointer to an utf8 string
/// * `size` and `alignment` must satisfy the Rust `Layout` type requirements
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_request_response_set_response_payload_type_details(
    service_builder_handle: iox2_service_builder_request_response_h_ref,
    type_variant: iox2_type_variant_e,
    type_name_str: *const c_char,
    type_name_len: c_size_t,
    size: c_size_t,
    alignment: c_size_t,
) -> c_int {
    service_builder_handle.assert_non_null();

    let value =
        match create_type_details(type_variant, type_name_str, type_name_len, size, alignment) {
            Ok(v) => v,
            Err(e) => return e,
        };

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_request_response(
                service_builder.__internal_set_response_payload_type_details(&value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_local_request_response(
                service_builder.__internal_set_response_payload_type_details(&value),
            ));
        }
    }

    IOX2_OK
}

/// Enables/disables fire and forget requests
///
/// # Safety
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_request_response_h_ref`]
///   obtained by
///   [`iox2_service_builder_request_response`](crate::iox2_service_builder_request_response).
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_request_response_enable_fire_and_forget_requests(
    service_builder_handle: iox2_service_builder_request_response_h_ref,
    value: bool,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_request_response(
                service_builder.enable_fire_and_forget_requests(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_local_request_response(
                service_builder.enable_fire_and_forget_requests(value),
            ));
        }
    }
}

/// Enables/disables safe overflow for requests
///
/// # Safety
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_request_response_h_ref`]
///   obtained by
///   [`iox2_service_builder_request_response`](crate::iox2_service_builder_request_response).
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_request_response_enable_safe_overflow_for_requests(
    service_builder_handle: iox2_service_builder_request_response_h_ref,
    value: bool,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_request_response(
                service_builder.enable_safe_overflow_for_requests(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_local_request_response(
                service_builder.enable_safe_overflow_for_requests(value),
            ));
        }
    }
}

/// Enables/disables safe overflow for responses
///
/// # Safety
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_request_response_h_ref`]
///   obtained by
///   [`iox2_service_builder_request_response`](crate::iox2_service_builder_request_response).
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_request_response_enable_safe_overflow_for_responses(
    service_builder_handle: iox2_service_builder_request_response_h_ref,
    value: bool,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_request_response(
                service_builder.enable_safe_overflow_for_responses(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_local_request_response(
                service_builder.enable_safe_overflow_for_responses(value),
            ));
        }
    }
}

/// Sets the maximum amount of active requests a client can have
///
/// # Safety
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_request_response_h_ref`]
///   obtained by
///   [`iox2_service_builder_request_response`](crate::iox2_service_builder_request_response).
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_request_response_max_active_requests_per_client(
    service_builder_handle: iox2_service_builder_request_response_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_request_response(
                service_builder.max_active_requests_per_client(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_local_request_response(
                service_builder.max_active_requests_per_client(value),
            ));
        }
    }
}

/// Sets the maximum amount responses a client can borrow from a pending response
///
/// # Safety
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_request_response_h_ref`]
///   obtained by
///   [`iox2_service_builder_request_response`](crate::iox2_service_builder_request_response).
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_request_response_max_borrowed_responses_per_pending_response(
    service_builder_handle: iox2_service_builder_request_response_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_request_response(
                service_builder.max_borrowed_responses_per_pending_response(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_local_request_response(
                service_builder.max_borrowed_responses_per_pending_response(value),
            ));
        }
    }
}

/// Sets the maximum number of clients the service will support
///
/// # Safety
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_request_response_h_ref`]
///   obtained by
///   [`iox2_service_builder_request_response`](crate::iox2_service_builder_request_response).
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_request_response_max_clients(
    service_builder_handle: iox2_service_builder_request_response_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_request_response(
                service_builder.max_clients(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_local_request_response(
                service_builder.max_clients(value),
            ));
        }
    }
}

/// Sets the maximum number of requests a client can loan at the same time
///
/// # Safety
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_request_response_h_ref`]
///   obtained by
///   [`iox2_service_builder_request_response`](crate::iox2_service_builder_request_response).
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_request_response_max_loaned_requests(
    service_builder_handle: iox2_service_builder_request_response_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_request_response(
                service_builder.max_loaned_requests(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_local_request_response(
                service_builder.max_loaned_requests(value),
            ));
        }
    }
}

/// Sets the maximum number of nodes that can open the service
///
/// # Safety
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_request_response_h_ref`]
///   obtained by
///   [`iox2_service_builder_request_response`](crate::iox2_service_builder_request_response).
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_request_response_set_max_nodes(
    service_builder_handle: iox2_service_builder_request_response_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_request_response(
                service_builder.max_nodes(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_local_request_response(
                service_builder.max_nodes(value),
            ));
        }
    }
}

/// Sets the maximum buffer size for responses on the client side
///
/// # Safety
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_request_response_h_ref`]
///   obtained by
///   [`iox2_service_builder_request_response`](crate::iox2_service_builder_request_response).
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_request_response_max_response_buffer_size(
    service_builder_handle: iox2_service_builder_request_response_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_request_response(
                service_builder.max_response_buffer_size(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_local_request_response(
                service_builder.max_response_buffer_size(value),
            ));
        }
    }
}

/// Sets the maximum number of servers the service will support
///
/// # Safety
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_request_response_h_ref`]
///   obtained by
///   [`iox2_service_builder_request_response`](crate::iox2_service_builder_request_response).
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_request_response_max_servers(
    service_builder_handle: iox2_service_builder_request_response_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_request_response(
                service_builder.max_servers(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_local_request_response(
                service_builder.max_servers(value),
            ));
        }
    }
}

/// Overrides the alignment of the provided request payload.
///
/// # Safety
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_request_response_h_ref`]
///   obtained by
///   [`iox2_service_builder_request_response`](crate::iox2_service_builder_request_response).
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_request_response_request_payload_alignment(
    service_builder_handle: iox2_service_builder_request_response_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_request_response(
                service_builder.request_payload_alignment(
                    Alignment::new(value).unwrap_or(Alignment::new_unchecked(8)),
                ),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_local_request_response(
                service_builder.request_payload_alignment(
                    Alignment::new(value).unwrap_or(Alignment::new_unchecked(8)),
                ),
            ));
        }
    }
}

/// Overrides the alignment of the provided response payload.
///
/// # Safety
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_request_response_h_ref`]
///   obtained by
///   [`iox2_service_builder_request_response`](crate::iox2_service_builder_request_response).
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_request_response_response_payload_alignment(
    service_builder_handle: iox2_service_builder_request_response_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_request_response(
                service_builder.response_payload_alignment(
                    Alignment::new(value).unwrap_or(Alignment::new_unchecked(8)),
                ),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);
            service_builder_struct.set(ServiceBuilderUnion::new_local_request_response(
                service_builder.response_payload_alignment(
                    Alignment::new(value).unwrap_or(Alignment::new_unchecked(8)),
                ),
            ));
        }
    }
}

/// Opens a request-response service or creates the service if it does not exist and returns a port factory to create servers and clients.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_request_response_h`]
///   obtained by [`iox2_service_builder_request_response`](crate::iox2_service_builder_request_response)
/// * `port_factory_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_port_factory_request_response_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `port_factory_handle_ptr` - An uninitialized or dangling [`iox2_port_factory_request_response_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_request_response_open_or_create_error_e`] otherwise.
///
/// # Safety
///
/// * The `service_builder_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_service_builder_t`](crate::iox2_service_builder_t) can be re-used with
///   a call to [`iox2_node_service_builder`](crate::iox2_node_service_builder)!
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_request_response_open_or_create(
    service_builder_handle: iox2_service_builder_request_response_h,
    port_factory_struct_ptr: *mut iox2_port_factory_request_response_t,
    port_factory_handle_ptr: *mut iox2_port_factory_request_response_h,
) -> c_int {
    iox2_service_builder_request_response_open_create_impl(
        service_builder_handle,
        port_factory_struct_ptr,
        port_factory_handle_ptr,
        |service_builder| service_builder.open_or_create(),
        |service_builder| service_builder.open_or_create(),
    )
}

/// Opens a request-response service or creates the service if it does not exist and returns a port factory to create servers and clients.
/// If the service does not exist, the provided arguments are stored inside the services, if the
/// service already exists, the provided attributes are considered as requirements.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_request_response_h`]
///   obtained by [`iox2_service_builder_request_response`](crate::iox2_service_builder_request_response)
/// * `port_factory_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_port_factory_request_response_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `attribute_verifier_handle` - An initialized valid handle to an [`iox2_attribute_verifier_h_ref`].
/// * `port_factory_handle_ptr` - An uninitialized or dangling [`iox2_port_factory_request_response_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_request_response_open_or_create_error_e`] otherwise.
///
/// # Safety
///
/// * The `service_builder_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_service_builder_t`](crate::iox2_service_builder_t) can be re-used with
///   a call to [`iox2_node_service_builder`](crate::iox2_node_service_builder)!
/// * The `attribute_verifier_handle` must be valid.
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_request_response_open_or_create_with_attributes(
    service_builder_handle: iox2_service_builder_request_response_h,
    attribute_verifier_handle: iox2_attribute_verifier_h_ref,
    port_factory_struct_ptr: *mut iox2_port_factory_request_response_t,
    port_factory_handle_ptr: *mut iox2_port_factory_request_response_h,
) -> c_int {
    let attribute_verifier_struct = &mut *attribute_verifier_handle.as_type();
    let attribute_verifier = &attribute_verifier_struct.value.as_ref().0;

    iox2_service_builder_request_response_open_create_impl(
        service_builder_handle,
        port_factory_struct_ptr,
        port_factory_handle_ptr,
        |service_builder| service_builder.open_or_create_with_attributes(attribute_verifier),
        |service_builder| service_builder.open_or_create_with_attributes(attribute_verifier),
    )
}

/// Opens a request-response service and returns a port factory to create servers and clients.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_request_response_h`]
///   obtained by [`iox2_service_builder_request_response`](crate::iox2_service_builder_request_response)
/// * `port_factory_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_port_factory_request_response_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `port_factory_handle_ptr` - An uninitialized or dangling [`iox2_port_factory_request_response_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_request_response_open_or_create_error_e`] otherwise. Note, only the errors annotated with `O_` are relevant.
///
/// # Safety
///
/// * The `service_builder_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_service_builder_t`](crate::iox2_service_builder_t) can be re-used with
///   a call to [`iox2_node_service_builder`](crate::iox2_node_service_builder)!
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_request_response_open(
    service_builder_handle: iox2_service_builder_request_response_h,
    port_factory_struct_ptr: *mut iox2_port_factory_request_response_t,
    port_factory_handle_ptr: *mut iox2_port_factory_request_response_h,
) -> c_int {
    iox2_service_builder_request_response_open_create_impl(
        service_builder_handle,
        port_factory_struct_ptr,
        port_factory_handle_ptr,
        |service_builder| service_builder.open(),
        |service_builder| service_builder.open(),
    )
}

/// Opens a request-response service and returns a port factory to create servers and clients.
/// The provided attributes are considered as requirements.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_request_response_h`]
///   obtained by [`iox2_service_builder_request_response`](crate::iox2_service_builder_request_response)
/// * `port_factory_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_port_factory_request_response_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `attribute_verifier_handle` - An initialized valid handle to an [`iox2_attribute_verifier_h_ref`].
/// * `port_factory_handle_ptr` - An uninitialized or dangling [`iox2_port_factory_request_response_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_request_response_open_or_create_error_e`] otherwise. Note, only the errors annotated with `O_` are relevant.
///
/// # Safety
///
/// * The `service_builder_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_service_builder_t`](crate::iox2_service_builder_t) can be re-used with
///   a call to [`iox2_node_service_builder`](crate::iox2_node_service_builder)!
/// * The `attribute_verifier_handle` must be valid.
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_request_response_open_with_attributes(
    service_builder_handle: iox2_service_builder_request_response_h,
    attribute_verifier_handle: iox2_attribute_verifier_h_ref,
    port_factory_struct_ptr: *mut iox2_port_factory_request_response_t,
    port_factory_handle_ptr: *mut iox2_port_factory_request_response_h,
) -> c_int {
    let attribute_verifier_struct = &mut *attribute_verifier_handle.as_type();
    let attribute_verifier = &attribute_verifier_struct.value.as_ref().0;

    iox2_service_builder_request_response_open_create_impl(
        service_builder_handle,
        port_factory_struct_ptr,
        port_factory_handle_ptr,
        |service_builder| service_builder.open_with_attributes(attribute_verifier),
        |service_builder| service_builder.open_with_attributes(attribute_verifier),
    )
}

/// Creates a request-response service and returns a port factory to create servers and clients.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_request_response_h`]
///   obtained by [`iox2_service_builder_request_response`](crate::iox2_service_builder_request_response)
/// * `port_factory_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_port_factory_request_response_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `port_factory_handle_ptr` - An uninitialized or dangling [`iox2_port_factory_request_response_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_request_response_open_or_create_error_e`] otherwise. Note, only the errors annotated with `C_` are relevant.
///
/// # Safety
///
/// * The `service_builder_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_service_builder_t`](crate::iox2_service_builder_t) can be re-used with
///   a call to [`iox2_node_service_builder`](crate::iox2_node_service_builder)!
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_request_response_create(
    service_builder_handle: iox2_service_builder_request_response_h,
    port_factory_struct_ptr: *mut iox2_port_factory_request_response_t,
    port_factory_handle_ptr: *mut iox2_port_factory_request_response_h,
) -> c_int {
    iox2_service_builder_request_response_open_create_impl(
        service_builder_handle,
        port_factory_struct_ptr,
        port_factory_handle_ptr,
        |service_builder| service_builder.create(),
        |service_builder| service_builder.create(),
    )
}

/// Creates a request-response service and returns a port factory to create servers and clients.
/// The provided arguments are stored inside the services.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_request_response_h`]
///   obtained by [`iox2_service_builder_request_response`](crate::iox2_service_builder_request_response)
/// * `port_factory_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_port_factory_request_response_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `attribute_specifier_handle` - An initialized valid handle to an [`iox2_attribute_specifier_h_ref`].
/// * `port_factory_handle_ptr` - An uninitialized or dangling [`iox2_port_factory_request_response_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_request_response_open_or_create_error_e`] otherwise. Note, only the errors annotated with `C_` are relevant.
///
/// # Safety
///
/// * The `service_builder_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_service_builder_t`](crate::iox2_service_builder_t) can be re-used with
///   a call to [`iox2_node_service_builder`](crate::iox2_node_service_builder)!
/// * The `attribute_specifier_handle` must be valid.
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_request_response_create_with_attributes(
    service_builder_handle: iox2_service_builder_request_response_h,
    attribute_specifier_handle: iox2_attribute_specifier_h_ref,
    port_factory_struct_ptr: *mut iox2_port_factory_request_response_t,
    port_factory_handle_ptr: *mut iox2_port_factory_request_response_h,
) -> c_int {
    let attribute_specifier_struct = &mut *attribute_specifier_handle.as_type();
    let attribute_specifier = &attribute_specifier_struct.value.as_ref().0;

    iox2_service_builder_request_response_open_create_impl(
        service_builder_handle,
        port_factory_struct_ptr,
        port_factory_handle_ptr,
        |service_builder| service_builder.create_with_attributes(attribute_specifier),
        |service_builder| service_builder.create_with_attributes(attribute_specifier),
    )
}

unsafe fn iox2_service_builder_request_response_open_create_impl<E: IntoCInt>(
    service_builder_handle: iox2_service_builder_request_response_h,
    port_factory_struct_ptr: *mut iox2_port_factory_request_response_t,
    port_factory_handle_ptr: *mut iox2_port_factory_request_response_h,
    func_ipc: impl FnOnce(
        Builder<PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi, crate::IpcService>,
    ) -> Result<
        PortFactory<crate::IpcService, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
        E,
    >,
    func_local: impl FnOnce(
        Builder<PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi, crate::LocalService>,
    ) -> Result<
        PortFactory<crate::LocalService, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
        E,
    >,
) -> c_int {
    service_builder_handle.assert_non_null();
    debug_assert!(!port_factory_handle_ptr.is_null());

    let init_port_factory_struct_ptr =
        |port_factory_struct_ptr: *mut iox2_port_factory_request_response_t| {
            let mut port_factory_struct_ptr = port_factory_struct_ptr;
            fn no_op(_: *mut iox2_port_factory_request_response_t) {}
            let mut deleter: fn(*mut iox2_port_factory_request_response_t) = no_op;
            if port_factory_struct_ptr.is_null() {
                port_factory_struct_ptr = iox2_port_factory_request_response_t::alloc();
                deleter = iox2_port_factory_request_response_t::dealloc;
            }
            debug_assert!(!port_factory_struct_ptr.is_null());

            (port_factory_struct_ptr, deleter)
        };

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };
    let service_type = service_builder_struct.service_type;
    let service_builder = service_builder_struct
        .value
        .as_option_mut()
        .take()
        .unwrap_or_else(|| {
            panic!("Trying to use an invalid 'iox2_service_builder_request_response_h'!");
        });
    (service_builder_struct.deleter)(service_builder_struct);

    match service_type {
        iox2_service_type_e::IPC => {
            let service_builder = ManuallyDrop::into_inner(service_builder.ipc);
            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);

            match func_ipc(service_builder) {
                Ok(port_factory) => {
                    let (port_factory_struct_ptr, deleter) =
                        init_port_factory_struct_ptr(port_factory_struct_ptr);
                    (*port_factory_struct_ptr).init(
                        service_type,
                        PortFactoryRequestResponseUnion::new_ipc(port_factory),
                        deleter,
                    );
                    *port_factory_handle_ptr = (*port_factory_struct_ptr).as_handle();
                }
                Err(error) => {
                    return error.into_c_int();
                }
            }
        }
        iox2_service_type_e::LOCAL => {
            let service_builder = ManuallyDrop::into_inner(service_builder.local);
            let service_builder = ManuallyDrop::into_inner(service_builder.request_response);

            match func_local(service_builder) {
                Ok(port_factory) => {
                    let (port_factory_struct_ptr, deleter) =
                        init_port_factory_struct_ptr(port_factory_struct_ptr);
                    (*port_factory_struct_ptr).init(
                        service_type,
                        PortFactoryRequestResponseUnion::new_local(port_factory),
                        deleter,
                    );
                    *port_factory_handle_ptr = (*port_factory_struct_ptr).as_handle();
                }
                Err(error) => {
                    return error.into_c_int();
                }
            }
        }
    }

    IOX2_OK
}
