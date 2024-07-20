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

// BEGIN type definition

use std::ffi::{c_int, c_void};

use iceoryx2::service::static_config::messaging_pattern;

use crate::{
    iox2_callback_progression_e, iox2_config_ptr, iox2_node_name_ptr, iox2_node_name_storage_t,
    iox2_service_name_ptr, IOX2_OK,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_service_type_e {
    LOCAL,
    IPC,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_messaging_pattern_e {
    PUBLISH_SUBSCRIBE = 0,
    EVENT,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_service_details_error_e {
    FAILED_TO_OPEN_STATIC_SERVICE_INFO = IOX2_OK as isize + 1,
    FAILED_TO_READ_STATIC_SERVICE_INFO,
    FAILED_TO_DESERIALIZE_STATIC_SERVICE_INFO,
    SERVICE_IN_INCONSISTENT_STATE,
    VERSION_MISMATCH,
    INTERNAL_ERROR,
    FAILED_TO_ACQUIRE_NODE_STATE,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_service_list_error_e {
    INSUFFICIENT_PERMISSIONS = IOX2_OK as isize + 1,
    INTERNAL_ERROR,
}

pub type iox2_service_list_callback =
    extern "C" fn(iox2_node_name_ptr) -> iox2_callback_progression_e;
pub type iox2_service_list_context = *const c_void;

// END type definition

#[no_mangle]
pub unsafe extern "C" fn iox2_service_does_exist(
    service_type: iox2_service_type_e,
    service_name: iox2_service_name_ptr,
    config: iox2_config_ptr,
    messaging_pattern: iox2_messaging_pattern_e,
    does_exist: *mut bool,
) -> c_int {
    todo!()
}

#[no_mangle]
pub unsafe extern "C" fn iox2_service_details(
    service_type: iox2_service_type_e,
    service_name: iox2_service_name_ptr,
    config: iox2_config_ptr,
    messaging_pattern: iox2_messaging_pattern_e,
    service_details: iox2_node_name_ptr,
) -> c_int {
    todo!()
}

#[no_mangle]
pub unsafe extern "C" fn iox2_service_list(
    config_ptr: iox2_config_ptr,
    callback: iox2_service_list_callback,
    callback_ctx: iox2_service_list_context,
) -> c_int {
    todo!()
}
