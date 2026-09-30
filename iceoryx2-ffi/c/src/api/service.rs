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

use core::ffi::{c_char, c_int};

use iceoryx2::service::{
    Service, ServiceDetails, ServiceDetailsError, ServiceListError,
    messaging_pattern::MessagingPattern,
};
use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_bb_elementary_traits::AsCStr;
use iceoryx2_ffi_macros::CStrRepr;

use crate::{
    IOX2_OK, iox2_callback_context, iox2_callback_progression_e, iox2_config_ptr,
    iox2_service_name_ptr, iox2_static_config_t,
};

use super::IntoCInt;

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum iox2_service_type_e {
    LOCAL,
    IPC,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_messaging_pattern_e {
    PUBLISH_SUBSCRIBE = 0,
    EVENT,
    REQUEST_RESPONSE,
    BLACKBOARD,
}

pub(crate) type IpcService = iceoryx2::prelude::ipc_threadsafe::Service;
pub(crate) type LocalService = iceoryx2::prelude::local_threadsafe::Service;

impl From<iox2_messaging_pattern_e> for MessagingPattern {
    fn from(value: iox2_messaging_pattern_e) -> Self {
        match value {
            iox2_messaging_pattern_e::EVENT => MessagingPattern::Event,
            iox2_messaging_pattern_e::PUBLISH_SUBSCRIBE => MessagingPattern::PublishSubscribe,
            iox2_messaging_pattern_e::REQUEST_RESPONSE => MessagingPattern::RequestResponse,
            iox2_messaging_pattern_e::BLACKBOARD => MessagingPattern::Blackboard,
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
            iceoryx2::service::static_config::messaging_pattern::MessagingPattern::RequestResponse(_) => {
                iox2_messaging_pattern_e::REQUEST_RESPONSE
            }
            iceoryx2::service::static_config::messaging_pattern::MessagingPattern::Blackboard(_) => {
                iox2_messaging_pattern_e::BLACKBOARD
            }
            _ => unreachable!()
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_service_details_error_e {
    FAILED_TO_OPEN_STATIC_SERVICE_INFO = IOX2_OK as isize + 1,
    FAILED_TO_READ_STATIC_SERVICE_INFO,
    FAILED_TO_DESERIALIZE_STATIC_SERVICE_INFO,
    SERVICE_IN_INCONSISTENT_STATE,
    VERSION_MISMATCH,
    INTERNAL_ERROR,
    FAILED_TO_ACQUIRE_NODE_STATE,
    INTERRUPT,
    INSUFFICIENT_PERMISSIONS,
}

impl IntoCInt for ServiceDetailsError {
    fn into_c_int(self) -> c_int {
        (match self {
            ServiceDetailsError::FailedToOpenStaticServiceInfo => {
                iox2_service_details_error_e::FAILED_TO_OPEN_STATIC_SERVICE_INFO
            }
            ServiceDetailsError::Interrupt => iox2_service_details_error_e::INTERRUPT,
            ServiceDetailsError::InsufficientPermissions => {
                iox2_service_details_error_e::INSUFFICIENT_PERMISSIONS
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
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_service_list_error_e {
    INSUFFICIENT_PERMISSIONS = IOX2_OK as isize + 1,
    INTERNAL_ERROR,
    FAILED_TO_OPEN_STATIC_SERVICE_INFO,
    FAILED_TO_READ_STATIC_SERVICE_INFO,
    FAILED_TO_DESERIALIZE_STATIC_SERVICE_INFO,
    SERVICE_IN_INCONSISTENT_STATE,
    VERSION_MISMATCH,
    FAILED_TO_ACQUIRE_NODE_STATE,
    INTERRUPT,
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

impl From<ServiceDetailsError> for iox2_service_list_error_e {
    fn from(error: ServiceDetailsError) -> Self {
        match error {
            ServiceDetailsError::FailedToOpenStaticServiceInfo => {
                iox2_service_list_error_e::FAILED_TO_OPEN_STATIC_SERVICE_INFO
            }
            ServiceDetailsError::FailedToReadStaticServiceInfo => {
                iox2_service_list_error_e::FAILED_TO_READ_STATIC_SERVICE_INFO
            }
            ServiceDetailsError::FailedToDeserializeStaticServiceInfo => {
                iox2_service_list_error_e::FAILED_TO_DESERIALIZE_STATIC_SERVICE_INFO
            }
            ServiceDetailsError::ServiceInInconsistentState => {
                iox2_service_list_error_e::SERVICE_IN_INCONSISTENT_STATE
            }
            ServiceDetailsError::VersionMismatch => iox2_service_list_error_e::VERSION_MISMATCH,
            ServiceDetailsError::FailedToAcquireNodeState => {
                iox2_service_list_error_e::FAILED_TO_ACQUIRE_NODE_STATE
            }
            ServiceDetailsError::Interrupt => iox2_service_list_error_e::INTERRUPT,
            ServiceDetailsError::InsufficientPermissions => {
                iox2_service_list_error_e::INSUFFICIENT_PERMISSIONS
            }
            ServiceDetailsError::InternalError => iox2_service_list_error_e::INTERNAL_ERROR,
        }
    }
}

pub type iox2_service_list_callback = extern "C" fn(
    *const iox2_static_config_t,
    iox2_callback_context,
) -> iox2_callback_progression_e;

// END type definition

// BEGIN C API

/// Returns a string literal describing the provided [`iox2_service_details_error_e`].
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_service_details_error_string(
    error: iox2_service_details_error_e,
) -> *const c_char {
    error.as_const_cstr().as_ptr() as *const c_char
}

/// Returns a string literal describing the provided [`iox2_service_list_error_e`].
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_service_list_error_string(
    error: iox2_service_list_error_e,
) -> *const c_char {
    error.as_const_cstr().as_ptr() as *const c_char
}

/// Checks if a specified service exists. If the service exists `does_exist` will contain
/// true, otherwise false after the call. On error it returns `iox2_service_details_error_e`, on
/// success `IOX2_OK`.
///
/// # Safety
///
/// * The `service_name` must be valid and non-null
/// * The `config` must be valid and non-null
/// * The `does_exist` must be valid and non-null
#[unsafe(no_mangle)]
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
    unsafe {
        let config = &*config;
        let service_name = &*service_name;
        let messaging_pattern = messaging_pattern.into();

        let result = match service_type {
            iox2_service_type_e::IPC => {
                IpcService::does_exist(service_name, config, messaging_pattern)
            }
            iox2_service_type_e::LOCAL => {
                LocalService::does_exist(service_name, config, messaging_pattern)
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
/// * The `does_exist` must be valid and non-null
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_service_details(
    service_type: iox2_service_type_e,
    service_name: iox2_service_name_ptr,
    config: iox2_config_ptr,
    messaging_pattern: iox2_messaging_pattern_e,
    service_details: *mut iox2_static_config_t,
    does_exist: *mut bool,
) -> c_int {
    debug_assert!(!service_name.is_null());
    debug_assert!(!config.is_null());
    debug_assert!(!service_details.is_null());
    debug_assert!(!does_exist.is_null());
    unsafe {
        let config = &*config;
        let service_name = &*service_name;
        let messaging_pattern = messaging_pattern.into();

        match service_type {
            iox2_service_type_e::IPC => {
                match IpcService::details(service_name, config, messaging_pattern) {
                    Ok(None) => {
                        does_exist.write(false);
                        IOX2_OK
                    }
                    Err(e) => e.into_c_int(),
                    Ok(Some(v)) => {
                        service_details.write((&v.static_details).into());
                        does_exist.write(true);
                        IOX2_OK
                    }
                }
            }
            iox2_service_type_e::LOCAL => {
                match LocalService::details(service_name, config, messaging_pattern) {
                    Ok(None) => {
                        does_exist.write(false);
                        IOX2_OK
                    }
                    Err(e) => e.into_c_int(),
                    Ok(Some(v)) => {
                        service_details.write((&v.static_details).into());
                        does_exist.write(true);
                        IOX2_OK
                    }
                }
            }
        }
    }
}

fn list_callback<S: Service>(
    callback: iox2_service_list_callback,
    callback_ctx: iox2_callback_context,
    service_details: Result<ServiceDetails<S>, ServiceDetailsError>,
    details_error: &mut Option<ServiceDetailsError>,
) -> CallbackProgression {
    match service_details {
        Ok(details) => callback(&(&details.static_details).into(), callback_ctx).into(),
        Err(error) => {
            *details_error = Some(error);
            CallbackProgression::Stop
        }
    }
}

/// Iterates over the all accessible services and calls the provided callback for
/// every service with iox2_service_details as input argument.
/// Stops on the first failure to acquire service details without calling the callback
/// for that service. On error it returns `iox2_service_list_error_e`, otherwise IOX2_OK.
///
/// # Safety
///
/// * The `config` must be valid and non-null
/// * The `callback` must be valid and non-null
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_service_list(
    service_type: iox2_service_type_e,
    config_ptr: iox2_config_ptr,
    callback: iox2_service_list_callback,
    callback_ctx: iox2_callback_context,
) -> c_int {
    debug_assert!(!config_ptr.is_null());

    let mut details_error = None;
    let result = match service_type {
        iox2_service_type_e::IPC => IpcService::list(unsafe { &*config_ptr }, |service_details| {
            list_callback::<IpcService>(callback, callback_ctx, service_details, &mut details_error)
        }),
        iox2_service_type_e::LOCAL => {
            LocalService::list(unsafe { &*config_ptr }, |service_details| {
                list_callback::<LocalService>(
                    callback,
                    callback_ctx,
                    service_details,
                    &mut details_error,
                )
            })
        }
    };

    match result {
        Err(error) => error.into_c_int(),
        Ok(()) => match details_error {
            Some(error) => iox2_service_list_error_e::from(error) as c_int,
            None => IOX2_OK,
        },
    }
}

// END C API

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_list_error_preserves_details_reason_and_existing_values() {
        assert_eq!(
            iox2_service_list_error_e::INSUFFICIENT_PERMISSIONS as c_int,
            IOX2_OK + 1
        );
        assert_eq!(
            iox2_service_list_error_e::INTERNAL_ERROR as c_int,
            IOX2_OK + 2
        );

        let cases = [
            (
                ServiceDetailsError::FailedToOpenStaticServiceInfo,
                iox2_service_list_error_e::FAILED_TO_OPEN_STATIC_SERVICE_INFO,
            ),
            (
                ServiceDetailsError::FailedToReadStaticServiceInfo,
                iox2_service_list_error_e::FAILED_TO_READ_STATIC_SERVICE_INFO,
            ),
            (
                ServiceDetailsError::FailedToDeserializeStaticServiceInfo,
                iox2_service_list_error_e::FAILED_TO_DESERIALIZE_STATIC_SERVICE_INFO,
            ),
            (
                ServiceDetailsError::ServiceInInconsistentState,
                iox2_service_list_error_e::SERVICE_IN_INCONSISTENT_STATE,
            ),
            (
                ServiceDetailsError::VersionMismatch,
                iox2_service_list_error_e::VERSION_MISMATCH,
            ),
            (
                ServiceDetailsError::FailedToAcquireNodeState,
                iox2_service_list_error_e::FAILED_TO_ACQUIRE_NODE_STATE,
            ),
            (
                ServiceDetailsError::Interrupt,
                iox2_service_list_error_e::INTERRUPT,
            ),
            (
                ServiceDetailsError::InsufficientPermissions,
                iox2_service_list_error_e::INSUFFICIENT_PERMISSIONS,
            ),
            (
                ServiceDetailsError::InternalError,
                iox2_service_list_error_e::INTERNAL_ERROR,
            ),
        ];
        for (reason, expected) in cases {
            assert_eq!(
                iox2_service_list_error_e::from(reason) as c_int,
                expected as c_int
            );
        }
    }

    #[test]
    fn failed_service_details_stop_without_calling_foreign_callback() {
        extern "C" fn callback(
            _: *const iox2_static_config_t,
            context: iox2_callback_context,
        ) -> iox2_callback_progression_e {
            // SAFETY: The test provides a valid, exclusive mutable bool pointer.
            unsafe { *context.cast::<bool>() = true };
            iox2_callback_progression_e::CONTINUE
        }

        let mut called = false;
        let context = (&mut called as *mut bool).cast();
        let error = ServiceDetailsError::VersionMismatch;
        let mut details_error = None;
        assert_eq!(
            list_callback::<IpcService>(callback, context, Err(error), &mut details_error),
            CallbackProgression::Stop
        );
        assert_eq!(details_error, Some(error));
        details_error = None;
        assert_eq!(
            list_callback::<LocalService>(callback, context, Err(error), &mut details_error),
            CallbackProgression::Stop
        );
        assert_eq!(details_error, Some(error));
        assert!(!called);
    }
}
