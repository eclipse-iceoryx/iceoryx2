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

use crate::api::{
    c_size_t, iox2_port_factory_pub_sub_h, iox2_port_factory_pub_sub_t,
    iox2_service_builder_pub_sub_h, iox2_service_builder_pub_sub_h_ref, iox2_service_type_e,
    AssertNonNullHandle, HandleToType, IntoCInt, PayloadFfi, PortFactoryPubSubUnion,
    ServiceBuilderUnion, UserHeaderFfi, IOX2_OK,
};

use iceoryx2::prelude::*;
use iceoryx2::service::builder::publish_subscribe::{
    Builder, PublishSubscribeCreateError, PublishSubscribeOpenError,
    PublishSubscribeOpenOrCreateError,
};
use iceoryx2::service::port_factory::publish_subscribe::PortFactory;
use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use iceoryx2_bb_log::fatal_panic;

use core::ffi::{c_char, c_int};
use core::mem::ManuallyDrop;
use core::{slice, str};
use std::alloc::Layout;

// BEGIN types definition

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_pub_sub_open_or_create_error_e {
    O_DOES_NOT_EXIST = IOX2_OK as isize + 1,
    O_INTERNAL_FAILURE,
    O_INCOMPATIBLE_TYPES,
    O_INCOMPATIBLE_MESSAGING_PATTERN,
    O_INCOMPATIBLE_ATTRIBUTES,
    O_DOES_NOT_SUPPORT_REQUESTED_MIN_BUFFER_SIZE,
    O_DOES_NOT_SUPPORT_REQUESTED_MIN_HISTORY_SIZE,
    O_DOES_NOT_SUPPORT_REQUESTED_MIN_SUBSCRIBER_BORROWED_SAMPLES,
    O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_PUBLISHERS,
    O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_SUBSCRIBERS,
    O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NODES,
    O_INCOMPATIBLE_OVERFLOW_BEHAVIOR,
    O_INSUFFICIENT_PERMISSIONS,
    O_SERVICE_IN_CORRUPTED_STATE,
    O_HANGS_IN_CREATION,
    O_EXCEEDS_MAX_NUMBER_OF_NODES,
    O_IS_MARKED_FOR_DESTRUCTION,
    C_SERVICE_IN_CORRUPTED_STATE,
    C_SUBSCRIBER_BUFFER_MUST_BE_LARGER_THAN_HISTORY_SIZE,
    C_ALREADY_EXISTS,
    C_INSUFFICIENT_PERMISSIONS,
    C_INTERNAL_FAILURE,
    C_IS_BEING_CREATED_BY_ANOTHER_INSTANCE,
    C_OLD_CONNECTION_STILL_ACTIVE,
    C_HANGS_IN_CREATION,
}

impl IntoCInt for PublishSubscribeOpenError {
    fn into_c_int(self) -> c_int {
        (match self {
            PublishSubscribeOpenError::DoesNotExist => iox2_pub_sub_open_or_create_error_e::O_DOES_NOT_EXIST,
         PublishSubscribeOpenError::InternalFailure => {
             iox2_pub_sub_open_or_create_error_e::O_INTERNAL_FAILURE
         }
         PublishSubscribeOpenError::IncompatibleTypes => {
             iox2_pub_sub_open_or_create_error_e::O_INCOMPATIBLE_TYPES
         }
         PublishSubscribeOpenError::IncompatibleMessagingPattern => {
             iox2_pub_sub_open_or_create_error_e::O_INCOMPATIBLE_MESSAGING_PATTERN
         }
         PublishSubscribeOpenError::IncompatibleAttributes => {
             iox2_pub_sub_open_or_create_error_e::O_INCOMPATIBLE_ATTRIBUTES
         }
         PublishSubscribeOpenError::DoesNotSupportRequestedMinBufferSize => {
             iox2_pub_sub_open_or_create_error_e::O_DOES_NOT_SUPPORT_REQUESTED_MIN_BUFFER_SIZE
         }
         PublishSubscribeOpenError::DoesNotSupportRequestedMinHistorySize => {
             iox2_pub_sub_open_or_create_error_e::O_DOES_NOT_SUPPORT_REQUESTED_MIN_HISTORY_SIZE
         }
         PublishSubscribeOpenError::DoesNotSupportRequestedMinSubscriberBorrowedSamples => {
             iox2_pub_sub_open_or_create_error_e::O_DOES_NOT_SUPPORT_REQUESTED_MIN_SUBSCRIBER_BORROWED_SAMPLES
         }
         PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfPublishers => {
             iox2_pub_sub_open_or_create_error_e::O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_PUBLISHERS
         }
         PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfSubscribers => {
             iox2_pub_sub_open_or_create_error_e::O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_SUBSCRIBERS
         }
         PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfNodes => {
             iox2_pub_sub_open_or_create_error_e::O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NODES
         }
         PublishSubscribeOpenError::IncompatibleOverflowBehavior => {
             iox2_pub_sub_open_or_create_error_e::O_INCOMPATIBLE_OVERFLOW_BEHAVIOR
         }
         PublishSubscribeOpenError::InsufficientPermissions => {
             iox2_pub_sub_open_or_create_error_e::O_INSUFFICIENT_PERMISSIONS
         }
         PublishSubscribeOpenError::ServiceInCorruptedState => {
             iox2_pub_sub_open_or_create_error_e::O_SERVICE_IN_CORRUPTED_STATE
         }
         PublishSubscribeOpenError::HangsInCreation => {
             iox2_pub_sub_open_or_create_error_e::O_HANGS_IN_CREATION
         }
         PublishSubscribeOpenError::ExceedsMaxNumberOfNodes => {
             iox2_pub_sub_open_or_create_error_e::O_EXCEEDS_MAX_NUMBER_OF_NODES
         }
         PublishSubscribeOpenError::IsMarkedForDestruction => {
             iox2_pub_sub_open_or_create_error_e::O_IS_MARKED_FOR_DESTRUCTION
         }
        }) as c_int
    }
}

impl IntoCInt for PublishSubscribeCreateError {
    fn into_c_int(self) -> c_int {
        (match self {
            PublishSubscribeCreateError::ServiceInCorruptedState => {
                iox2_pub_sub_open_or_create_error_e::C_SERVICE_IN_CORRUPTED_STATE
            }
            PublishSubscribeCreateError::SubscriberBufferMustBeLargerThanHistorySize => {
                iox2_pub_sub_open_or_create_error_e::C_SUBSCRIBER_BUFFER_MUST_BE_LARGER_THAN_HISTORY_SIZE
            }
            PublishSubscribeCreateError::AlreadyExists => iox2_pub_sub_open_or_create_error_e::C_ALREADY_EXISTS,
          PublishSubscribeCreateError::InsufficientPermissions => {
             iox2_pub_sub_open_or_create_error_e::C_INSUFFICIENT_PERMISSIONS
         }
            PublishSubscribeCreateError::InternalFailure => {
                iox2_pub_sub_open_or_create_error_e::C_INTERNAL_FAILURE
            }
            PublishSubscribeCreateError::IsBeingCreatedByAnotherInstance => {
                iox2_pub_sub_open_or_create_error_e::C_IS_BEING_CREATED_BY_ANOTHER_INSTANCE
            }
         PublishSubscribeCreateError::HangsInCreation => {
             iox2_pub_sub_open_or_create_error_e::C_HANGS_IN_CREATION
         }
        }) as c_int
    }
}

impl IntoCInt for PublishSubscribeOpenOrCreateError {
    fn into_c_int(self) -> c_int {
        match self {
            PublishSubscribeOpenOrCreateError::PublishSubscribeOpenError(error) => {
                error.into_c_int()
            }
            PublishSubscribeOpenOrCreateError::PublishSubscribeCreateError(error) => {
                error.into_c_int()
            }
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_type_variant_e {
    FIXED_SIZE,
    DYNAMIC,
}

impl From<&TypeVariant> for iox2_type_variant_e {
    fn from(value: &TypeVariant) -> Self {
        match value {
            TypeVariant::Dynamic => iox2_type_variant_e::DYNAMIC,
            TypeVariant::FixedSize => iox2_type_variant_e::FIXED_SIZE,
        }
    }
}

impl From<iox2_type_variant_e> for TypeVariant {
    fn from(value: iox2_type_variant_e) -> Self {
        const DYNAMIC: usize = iox2_type_variant_e::DYNAMIC as usize;
        const FIXED_SIZE: usize = iox2_type_variant_e::FIXED_SIZE as usize;

        match value as usize {
            DYNAMIC => TypeVariant::Dynamic,
            FIXED_SIZE => TypeVariant::FixedSize,
            e => fatal_panic!("Invalid iox2_type_variant_e value {}", e),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_type_detail_error_e {
    INVALID_TYPE_NAME = IOX2_OK as isize + 1,
    INVALID_SIZE_OR_ALIGNMENT_VALUE,
}

// END type definition

// BEGIN C API

/// Sets the user header type details for the builder
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_pub_sub_h_ref`]
///   obtained by [`iox2_service_builder_pub_sub`](crate::iox2_service_builder_pub_sub).
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
pub unsafe extern "C" fn iox2_service_builder_pub_sub_set_user_header_type_details(
    service_builder_handle: iox2_service_builder_pub_sub_h_ref,
    type_variant: iox2_type_variant_e,
    type_name_str: *const c_char,
    type_name_len: c_size_t,
    size: c_size_t,
    alignment: c_size_t,
) -> c_int {
    service_builder_handle.assert_non_null();
    debug_assert!(!type_name_str.is_null());

    let type_name = slice::from_raw_parts(type_name_str as _, type_name_len as _);

    let type_name = if let Ok(type_name) = str::from_utf8(type_name) {
        type_name.to_string()
    } else {
        return iox2_type_detail_error_e::INVALID_TYPE_NAME as c_int;
    };

    match Layout::from_size_align(size, alignment) {
        Ok(_) => (),
        Err(_) => return iox2_type_detail_error_e::INVALID_SIZE_OR_ALIGNMENT_VALUE as c_int,
    }

    let value = TypeDetail {
        variant: type_variant.into(),
        type_name,
        size,
        alignment,
    };

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_pub_sub(
                service_builder.__internal_set_user_header_type_details(&value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builder_struct.set(ServiceBuilderUnion::new_local_pub_sub(
                service_builder.__internal_set_user_header_type_details(&value),
            ));
        }
    }

    IOX2_OK
}

/// Sets the payload type details for the builder
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_pub_sub_h_ref`]
///   obtained by [`iox2_service_builder_pub_sub`](crate::iox2_service_builder_pub_sub).
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
pub unsafe extern "C" fn iox2_service_builder_pub_sub_set_payload_type_details(
    service_builder_handle: iox2_service_builder_pub_sub_h_ref,
    type_variant: iox2_type_variant_e,
    type_name_str: *const c_char,
    type_name_len: c_size_t,
    size: c_size_t,
    alignment: c_size_t,
) -> c_int {
    service_builder_handle.assert_non_null();
    debug_assert!(!type_name_str.is_null());

    let type_name = slice::from_raw_parts(type_name_str as _, type_name_len as _);

    let type_name = if let Ok(type_name) = str::from_utf8(type_name) {
        type_name.to_string()
    } else {
        return iox2_type_detail_error_e::INVALID_TYPE_NAME as c_int;
    };

    match Layout::from_size_align(size, alignment) {
        Ok(_) => (),
        Err(_) => return iox2_type_detail_error_e::INVALID_SIZE_OR_ALIGNMENT_VALUE as c_int,
    }

    let value = TypeDetail {
        variant: type_variant.into(),
        type_name,
        size,
        alignment,
    };

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_pub_sub(
                service_builder.__internal_set_payload_type_details(&value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builder_struct.set(ServiceBuilderUnion::new_local_pub_sub(
                service_builder.__internal_set_payload_type_details(&value),
            ));
        }
    }

    IOX2_OK
}

/// Sets the max nodes for the builder
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_pub_sub_h_ref`]
///   obtained by [`iox2_service_builder_pub_sub`](crate::iox2_service_builder_pub_sub).
/// * `value` - The value to set the max nodes to
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_pub_sub_set_max_nodes(
    service_builder_handle: iox2_service_builder_pub_sub_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_pub_sub(
                service_builder.max_nodes(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builder_struct.set(ServiceBuilderUnion::new_local_pub_sub(
                service_builder.max_nodes(value),
            ));
        }
    }
}

/// Sets the max publishers for the builder
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_pub_sub_h_ref`]
///   obtained by [`iox2_service_builder_pub_sub`](crate::iox2_service_builder_pub_sub).
/// * `value` - The value to set the max publishers to
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_pub_sub_set_max_publishers(
    service_builder_handle: iox2_service_builder_pub_sub_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_pub_sub(
                service_builder.max_publishers(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builder_struct.set(ServiceBuilderUnion::new_local_pub_sub(
                service_builder.max_publishers(value),
            ));
        }
    }
}

/// Sets the max subscribers for the builder
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_pub_sub_h_ref`]
///   obtained by [`iox2_service_builder_pub_sub`](crate::iox2_service_builder_pub_sub).
/// * `value` - The value to set the max subscribers to
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_pub_sub_set_max_subscribers(
    service_builder_handle: iox2_service_builder_pub_sub_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_pub_sub(
                service_builder.max_subscribers(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builder_struct.set(ServiceBuilderUnion::new_local_pub_sub(
                service_builder.max_subscribers(value),
            ));
        }
    }
}

/// Sets the payload alignment for the builder
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_pub_sub_h_ref`]
///   obtained by [`iox2_service_builder_pub_sub`](crate::iox2_service_builder_pub_sub).
/// * `value` - The value to set the payload alignment to
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_pub_sub_set_payload_alignment(
    service_builder_handle: iox2_service_builder_pub_sub_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_pub_sub(
                service_builder.payload_alignment(
                    Alignment::new(value).unwrap_or(Alignment::new_unchecked(8)),
                ),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builder_struct.set(ServiceBuilderUnion::new_local_pub_sub(
                service_builder.payload_alignment(
                    Alignment::new(value).unwrap_or(Alignment::new_unchecked(8)),
                ),
            ));
        }
    }
}

/// Sets the history size
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_pub_sub_h_ref`]
///   obtained by [`iox2_service_builder_pub_sub`](crate::iox2_service_builder_pub_sub).
/// * `value` - The value to set the history size to
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_pub_sub_set_history_size(
    service_builder_handle: iox2_service_builder_pub_sub_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_pub_sub(
                service_builder.history_size(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builder_struct.set(ServiceBuilderUnion::new_local_pub_sub(
                service_builder.history_size(value),
            ));
        }
    }
}

/// Sets the subscriber max buffer size
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_pub_sub_h_ref`]
///   obtained by [`iox2_service_builder_pub_sub`](crate::iox2_service_builder_pub_sub).
/// * `value` - The value to set the subscriber max buffer size to
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_pub_sub_set_subscriber_max_buffer_size(
    service_builder_handle: iox2_service_builder_pub_sub_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_pub_sub(
                service_builder.subscriber_max_buffer_size(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builder_struct.set(ServiceBuilderUnion::new_local_pub_sub(
                service_builder.subscriber_max_buffer_size(value),
            ));
        }
    }
}

/// Sets the subscriber max borrowed samples
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_pub_sub_h_ref`]
///   obtained by [`iox2_service_builder_pub_sub`](crate::iox2_service_builder_pub_sub).
/// * `value` - The value to set the subscriber max borrowed samples to
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_pub_sub_set_subscriber_max_borrowed_samples(
    service_builder_handle: iox2_service_builder_pub_sub_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_pub_sub(
                service_builder.subscriber_max_borrowed_samples(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builder_struct.set(ServiceBuilderUnion::new_local_pub_sub(
                service_builder.subscriber_max_borrowed_samples(value),
            ));
        }
    }
}

/// Enables/disables safe overflow for the service
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_pub_sub_h_ref`]
///   obtained by [`iox2_service_builder_pub_sub`](crate::iox2_service_builder_pub_sub).
/// * `value` - defines if safe overflow shall be enabled (true) or not (false)
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_pub_sub_set_enable_safe_overflow(
    service_builder_handle: iox2_service_builder_pub_sub_h_ref,
    value: bool,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_pub_sub(
                service_builder.enable_safe_overflow(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builder_struct.set(ServiceBuilderUnion::new_local_pub_sub(
                service_builder.enable_safe_overflow(value),
            ));
        }
    }
}

// TODO [#210] add all the other setter methods

/// Opens a publish-subscribe service or creates the service if it does not exist and returns a port factory to create publishers and subscribers.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_pub_sub_h`]
///   obtained by [`iox2_service_builder_pub_sub`](crate::iox2_service_builder_pub_sub)
/// * `port_factory_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_port_factory_pub_sub_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `port_factory_handle_ptr` - An uninitialized or dangling [`iox2_port_factory_pub_sub_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_pub_sub_open_or_create_error_e`] otherwise.
///
/// # Safety
///
/// * The `service_builder_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_service_builder_t`](crate::iox2_service_builder_t) can be re-used with
///   a call to [`iox2_node_service_builder`](crate::iox2_node_service_builder)!
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_pub_sub_open_or_create(
    service_builder_handle: iox2_service_builder_pub_sub_h,
    port_factory_struct_ptr: *mut iox2_port_factory_pub_sub_t,
    port_factory_handle_ptr: *mut iox2_port_factory_pub_sub_h,
) -> c_int {
    iox2_service_builder_pub_sub_open_create_impl(
        service_builder_handle,
        port_factory_struct_ptr,
        port_factory_handle_ptr,
        |service_builder| service_builder.open_or_create(),
        |service_builder| service_builder.open_or_create(),
    )
}

/// Opens a publish-subscribe service and returns a port factory to create publishers and subscribers.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_pub_sub_h`]
///   obtained by [`iox2_service_builder_pub_sub`](crate::iox2_service_builder_pub_sub)
/// * `port_factory_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_port_factory_pub_sub_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `port_factory_handle_ptr` - An uninitialized or dangling [`iox2_port_factory_pub_sub_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_pub_sub_open_or_create_error_e`] otherwise. Note, only the errors annotated with `O_` are relevant.
///
/// # Safety
///
/// * The `service_builder_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_service_builder_t`](crate::iox2_service_builder_t) can be re-used with
///   a call to [`iox2_node_service_builder`](crate::iox2_node_service_builder)!
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_pub_sub_open(
    service_builder_handle: iox2_service_builder_pub_sub_h,
    port_factory_struct_ptr: *mut iox2_port_factory_pub_sub_t,
    port_factory_handle_ptr: *mut iox2_port_factory_pub_sub_h,
) -> c_int {
    iox2_service_builder_pub_sub_open_create_impl(
        service_builder_handle,
        port_factory_struct_ptr,
        port_factory_handle_ptr,
        |service_builder| service_builder.open(),
        |service_builder| service_builder.open(),
    )
}

/// Creates a publish-subscribe service and returns a port factory to create publishers and subscribers.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_pub_sub_h`]
///   obtained by [`iox2_service_builder_pub_sub`](crate::iox2_service_builder_pub_sub)
/// * `port_factory_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_port_factory_pub_sub_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `port_factory_handle_ptr` - An uninitialized or dangling [`iox2_port_factory_pub_sub_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_pub_sub_open_or_create_error_e`] otherwise. Note, only the errors annotated with `C_` are relevant.
///
/// # Safety
///
/// * The `service_builder_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_service_builder_t`](crate::iox2_service_builder_t) can be re-used with
///   a call to [`iox2_node_service_builder`](crate::iox2_node_service_builder)!
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_pub_sub_create(
    service_builder_handle: iox2_service_builder_pub_sub_h,
    port_factory_struct_ptr: *mut iox2_port_factory_pub_sub_t,
    port_factory_handle_ptr: *mut iox2_port_factory_pub_sub_h,
) -> c_int {
    iox2_service_builder_pub_sub_open_create_impl(
        service_builder_handle,
        port_factory_struct_ptr,
        port_factory_handle_ptr,
        |service_builder| service_builder.create(),
        |service_builder| service_builder.create(),
    )
}

unsafe fn iox2_service_builder_pub_sub_open_create_impl<E: IntoCInt>(
    service_builder_handle: iox2_service_builder_pub_sub_h,
    port_factory_struct_ptr: *mut iox2_port_factory_pub_sub_t,
    port_factory_handle_ptr: *mut iox2_port_factory_pub_sub_h,
    func_ipc: impl FnOnce(
        Builder<PayloadFfi, UserHeaderFfi, ipc::Service>,
    ) -> Result<PortFactory<ipc::Service, PayloadFfi, UserHeaderFfi>, E>,
    func_local: impl FnOnce(
        Builder<PayloadFfi, UserHeaderFfi, local::Service>,
    ) -> Result<PortFactory<local::Service, PayloadFfi, UserHeaderFfi>, E>,
) -> c_int {
    service_builder_handle.assert_non_null();
    debug_assert!(!port_factory_handle_ptr.is_null());

    let init_port_factory_struct_ptr =
        |port_factory_struct_ptr: *mut iox2_port_factory_pub_sub_t| {
            let mut port_factory_struct_ptr = port_factory_struct_ptr;
            fn no_op(_: *mut iox2_port_factory_pub_sub_t) {}
            let mut deleter: fn(*mut iox2_port_factory_pub_sub_t) = no_op;
            if port_factory_struct_ptr.is_null() {
                port_factory_struct_ptr = iox2_port_factory_pub_sub_t::alloc();
                deleter = iox2_port_factory_pub_sub_t::dealloc;
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
            panic!("Trying to use an invalid 'iox2_service_builder_pub_sub_h'!");
        });
    (service_builder_struct.deleter)(service_builder_struct);

    match service_type {
        iox2_service_type_e::IPC => {
            let service_builder = ManuallyDrop::into_inner(service_builder.ipc);
            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);

            match func_ipc(service_builder) {
                Ok(port_factory) => {
                    let (port_factory_struct_ptr, deleter) =
                        init_port_factory_struct_ptr(port_factory_struct_ptr);
                    (*port_factory_struct_ptr).init(
                        service_type,
                        PortFactoryPubSubUnion::new_ipc(port_factory),
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
            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);

            match func_local(service_builder) {
                Ok(port_factory) => {
                    let (port_factory_struct_ptr, deleter) =
                        init_port_factory_struct_ptr(port_factory_struct_ptr);
                    (*port_factory_struct_ptr).init(
                        service_type,
                        PortFactoryPubSubUnion::new_local(port_factory),
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

// END C API
