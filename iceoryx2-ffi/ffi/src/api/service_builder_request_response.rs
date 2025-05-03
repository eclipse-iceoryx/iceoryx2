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

use iceoryx2::service::static_config::message_type_details::TypeDetail;

use crate::{
    api::{AssertNonNullHandle, HandleToType, ServiceBuilderUnion},
    iox2_service_type_e, iox2_type_detail_error_e, IOX2_OK,
};

use super::{c_size_t, iox2_service_builder_request_response_h_ref, iox2_type_variant_e};

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

    match Layout::from_size_align(size, alignment) {
        Ok(_) => (),
        Err(_) => return Err(iox2_type_detail_error_e::INVALID_SIZE_OR_ALIGNMENT_VALUE as c_int),
    }

    Ok(TypeDetail {
        variant: type_variant.into(),
        type_name,
        size,
        alignment,
    })
}

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
