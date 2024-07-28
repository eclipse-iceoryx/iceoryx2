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
#![allow(non_snake_case)]

// BEGIN type definition

use std::ffi::c_int;

use iceoryx2::service::{
    messaging_pattern::MessagingPattern, process_local, zero_copy, Service, ServiceDetails,
    ServiceDetailsError, ServiceListError,
};
use iceoryx2_bb_elementary::CallbackProgression;

use crate::{
    iox2_callback_context, iox2_callback_progression_e, iox2_config_ptr, iox2_service_name_ptr,
    iox2_static_config_t, IOX2_OK,
};

use super::IntoCInt;

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

impl From<iox2_messaging_pattern_e> for MessagingPattern {
    fn from(value: iox2_messaging_pattern_e) -> Self {
        match value {
            iox2_messaging_pattern_e::EVENT => MessagingPattern::Event,
            iox2_messaging_pattern_e::PUBLISH_SUBSCRIBE => MessagingPattern::PublishSubscribe,
        }
    }
}

impl From<&iceoryx2::service::static_config::messaging_pattern::MessagingPattern>
    for iox2_messaging_pattern_e
{
    fn from(value: &iceoryx2::service::static_config::messaging_pattern::MessagingPattern) -> Self {
        match value {
            iceoryx2::service::static_config::messaging_pattern::MessagingPattern::Event(_) => {
                iox2_messaging_pattern_e::EVENT
            }
            iceoryx2::service::static_config::messaging_pattern::MessagingPattern::PublishSubscribe(_) => {
                iox2_messaging_pattern_e::PUBLISH_SUBSCRIBE
            }
            _ => todo!()
        }
    }
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

impl IntoCInt for ServiceDetailsError {
    fn into_c_int(self) -> c_int {
        (match self {
            ServiceDetailsError::FailedToOpenStaticServiceInfo => {
                iox2_service_details_error_e::FAILED_TO_OPEN_STATIC_SERVICE_INFO
            }
            ServiceDetailsError::FailedToReadStaticServiceInfo => {
                iox2_service_details_error_e::FAILED_TO_READ_STATIC_SERVICE_INFO
            }
            ServiceDetailsError::FailedToDeserializeStaticServiceInfo => {
                iox2_service_details_error_e::FAILED_TO_DESERIALIZE_STATIC_SERVICE_INFO
            }
            ServiceDetailsError::ServiceInInconsistentState => {
                iox2_service_details_error_e::SERVICE_IN_INCONSISTENT_STATE
            }
            ServiceDetailsError::VersionMismatch => iox2_service_details_error_e::VERSION_MISMATCH,
            ServiceDetailsError::InternalError => iox2_service_details_error_e::INTERNAL_ERROR,
            ServiceDetailsError::FailedToAcquireNodeState => {
                iox2_service_details_error_e::FAILED_TO_ACQUIRE_NODE_STATE
            }
        }) as c_int
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_service_list_error_e {
    INSUFFICIENT_PERMISSIONS = IOX2_OK as isize + 1,
    INTERNAL_ERROR,
}

impl IntoCInt for ServiceListError {
    fn into_c_int(self) -> c_int {
        match self {
            ServiceListError::InternalError => iox2_service_list_error_e::INTERNAL_ERROR as _,
            ServiceListError::InsufficientPermissions => {
                iox2_service_list_error_e::INSUFFICIENT_PERMISSIONS as _
            }
        }
    }
}

pub type iox2_service_list_callback = extern "C" fn(
    *const iox2_static_config_t,
    iox2_callback_context,
) -> iox2_callback_progression_e;

// END type definition

/// Checks if a specified service exists. If the service exists `does_exist` will contain
/// true, otherwise false after the call. On error it returns `iox2_service_details_error_e`, on
/// success `IOX2_OK`.
///
/// # Safety
///
/// * The `service_name` must be valid and non-null
/// * The `config` must be valid and non-null
/// * The `does_exist` must be valid and non-null
#[no_mangle]
pub unsafe extern "C" fn iox2_service_does_exist(
    service_type: iox2_service_type_e,
    service_name: iox2_service_name_ptr,
    config: iox2_config_ptr,
    messaging_pattern: iox2_messaging_pattern_e,
    does_exist: *mut bool,
) -> c_int {
    debug_assert!(!service_name.is_null());
    debug_assert!(!config.is_null());
    debug_assert!(!does_exist.is_null());

    let config = &*config;
    let service_name = &*service_name;
    let messaging_pattern = messaging_pattern.into();

    let result = match service_type {
        iox2_service_type_e::IPC => {
            zero_copy::Service::does_exist(service_name, config, messaging_pattern)
        }
        iox2_service_type_e::LOCAL => {
            process_local::Service::does_exist(service_name, config, messaging_pattern)
        }
    };

    match result {
        Ok(value) => {
            *does_exist = value;
            IOX2_OK
        }
        Err(e) => e.into_c_int(),
    }
}

/// Acquires the service details of a specified service. If the service exists `service_details` will contain
/// the requested information, otherwise it is NULL. On error it returns `iox2_service_details_error_e`, on
/// success `IOX2_OK`.
///
/// # Safety
///
/// * The `service_name` must be valid and non-null
/// * The `config` must be valid and non-null
/// * The `service_details` must be valid and non-null
#[no_mangle]
pub unsafe extern "C" fn iox2_service_details(
    _service_type: iox2_service_type_e,
    service_name: iox2_service_name_ptr,
    config: iox2_config_ptr,
    _messaging_pattern: iox2_messaging_pattern_e,
    service_details: *mut iox2_static_config_t,
) -> c_int {
    debug_assert!(!service_name.is_null());
    debug_assert!(!config.is_null());
    debug_assert!(!service_details.is_null());

    todo!()
}

fn list_callback<S: Service>(
    callback: iox2_service_list_callback,
    callback_ctx: iox2_callback_context,
    service_details: &ServiceDetails<S>,
) -> CallbackProgression {
    callback(&(&service_details.static_details).into(), callback_ctx).into()
}

/// Iterates over the all accessible services and calls the provided callback for
/// every service with iox2_service_details as input argument.
/// On error it returns `iox2_service_list_error_e`, otherwise IOX2_OK.
///
/// # Safety
///
/// * The `config` must be valid and non-null
/// * The `callback` must be valid and non-null
#[no_mangle]
pub unsafe extern "C" fn iox2_service_list(
    service_type: iox2_service_type_e,
    config_ptr: iox2_config_ptr,
    callback: iox2_service_list_callback,
    callback_ctx: iox2_callback_context,
) -> c_int {
    debug_assert!(!config_ptr.is_null());

    let result = match service_type {
        iox2_service_type_e::IPC => zero_copy::Service::list(&*config_ptr, |service_details| {
            list_callback::<zero_copy::Service>(callback, callback_ctx, &service_details)
        }),
        iox2_service_type_e::LOCAL => {
            process_local::Service::list(&*config_ptr, |service_details| {
                list_callback::<process_local::Service>(callback, callback_ctx, &service_details)
            })
        }
    };

    match result {
        Ok(()) => IOX2_OK,
        Err(e) => e.into_c_int(),
    }
}
