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
    iox2_service_builder_pub_sub_h, iox2_service_builder_pub_sub_ref_h, iox2_service_type_e,
    HandleToType, IntoCInt, PortFactoryPubSubUnion, ServiceBuilderUnion,
};
use crate::IOX2_OK;

use iceoryx2::service::builder::publish_subscribe::{
    PublishSubscribeCreateError, PublishSubscribeOpenError, PublishSubscribeOpenOrCreateError,
};

use core::ffi::c_int;
use core::mem::ManuallyDrop;

// BEGIN types definition

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_pub_sub_open_or_create_error_e {
    O_DOES_NOT_EXIST = 1,
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
         PublishSubscribeCreateError::OldConnectionsStillActive => {
             iox2_pub_sub_open_or_create_error_e::C_OLD_CONNECTION_STILL_ACTIVE
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

// END type definition

// BEGIN C API

/// Sets the max publishers for the builder
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_pub_sub_ref_h`](crate::iox2_service_builder_pub_sub_ref_h)
///   obtained by [`iox2_service_builder_pub_sub`](crate::iox2_service_builder_pub_sub) and
///   casted by [`iox2_cast_service_builder_pub_sub_ref_h`](crate::iox2_cast_service_builder_pub_sub_ref_h).
/// * `value` - The value to set the max publishers to
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_pub_sub_set_max_publishers(
    service_builder_handle: iox2_service_builder_pub_sub_ref_h,
    value: c_size_t,
) {
    debug_assert!(!service_builder_handle.is_null());

    let service_builders_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builders_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builders_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builders_struct.set(ServiceBuilderUnion::new_ipc_pub_sub(
                service_builder.max_publishers(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builders_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builders_struct.set(ServiceBuilderUnion::new_local_pub_sub(
                service_builder.max_publishers(value),
            ));
        }
    }
}

/// Sets the max subscribers for the builder
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_pub_sub_ref_h`](crate::iox2_service_builder_pub_sub_ref_h)
///   obtained by [`iox2_service_builder_pub_sub`](crate::iox2_service_builder_pub_sub) and
///   casted by [`iox2_cast_service_builder_pub_sub_ref_h`](crate::iox2_cast_service_builder_pub_sub_ref_h).
/// * `value` - The value to set the max subscribers to
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_pub_sub_set_max_subscribers(
    service_builder_handle: iox2_service_builder_pub_sub_ref_h,
    value: c_size_t,
) {
    debug_assert!(!service_builder_handle.is_null());

    let service_builders_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builders_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builders_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builders_struct.set(ServiceBuilderUnion::new_ipc_pub_sub(
                service_builder.max_subscribers(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builders_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);
            service_builders_struct.set(ServiceBuilderUnion::new_local_pub_sub(
                service_builder.max_subscribers(value),
            ));
        }
    }
}

// TODO [#210] add all the other setter methods

/// Opens a publish-subscribe service or creates the service if it does not exist and returns a port factory to create publishers and subscribers.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_pub_sub_h`](crate::iox2_service_builder_pub_sub_h)
///   obtained by [`iox2_service_builder_pub_sub`](crate::iox2_service_builder_pub_sub)
/// * `port_factory_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_port_factory_pub_sub_t`](crate::iox2_port_factory_pub_sub_t). If it is a NULL pointer, the storage will be allocated on the heap.
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
    debug_assert!(!service_builder_handle.is_null());
    debug_assert!(!port_factory_handle_ptr.is_null());

    let service_builders_struct = unsafe { &mut *service_builder_handle.as_type() };

    let mut port_factory_struct_ptr = port_factory_struct_ptr;
    fn no_op(_: *mut iox2_port_factory_pub_sub_t) {}
    let mut deleter: fn(*mut iox2_port_factory_pub_sub_t) = no_op;
    if port_factory_struct_ptr.is_null() {
        port_factory_struct_ptr = iox2_port_factory_pub_sub_t::alloc();
        deleter = iox2_port_factory_pub_sub_t::dealloc;
    }
    debug_assert!(!port_factory_struct_ptr.is_null());

    let service_type = service_builders_struct.service_type;
    match service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builders_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);

            match service_builder.open_or_create() {
                Ok(port_factory) => {
                    (*port_factory_struct_ptr).init(
                        service_type,
                        PortFactoryPubSubUnion::new_ipc(port_factory),
                        deleter,
                    );
                }
                Err(error) => {
                    return error.into_c_int();
                }
            }
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builders_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);

            match service_builder.open_or_create() {
                Ok(port_factory) => {
                    (*port_factory_struct_ptr).init(
                        service_type,
                        PortFactoryPubSubUnion::new_local(port_factory),
                        deleter,
                    );
                }
                Err(error) => {
                    return error.into_c_int();
                }
            }
        }
    }

    *port_factory_handle_ptr = (*port_factory_struct_ptr).as_handle();

    IOX2_OK
}

/// Opens a publish-subscribe service and returns a port factory to create publishers and subscribers.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_pub_sub_h`](crate::iox2_service_builder_pub_sub_h)
///   obtained by [`iox2_service_builder_pub_sub`](crate::iox2_service_builder_pub_sub)
/// * `port_factory_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_port_factory_pub_sub_t`](crate::iox2_port_factory_pub_sub_t). If it is a NULL pointer, the storage will be allocated on the heap.
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
    debug_assert!(!service_builder_handle.is_null());
    debug_assert!(!port_factory_handle_ptr.is_null());

    let service_builders_struct = unsafe { &mut *service_builder_handle.as_type() };

    let mut port_factory_struct_ptr = port_factory_struct_ptr;
    fn no_op(_: *mut iox2_port_factory_pub_sub_t) {}
    let mut deleter: fn(*mut iox2_port_factory_pub_sub_t) = no_op;
    if port_factory_struct_ptr.is_null() {
        port_factory_struct_ptr = iox2_port_factory_pub_sub_t::alloc();
        deleter = iox2_port_factory_pub_sub_t::dealloc;
    }
    debug_assert!(!port_factory_struct_ptr.is_null());

    let service_type = service_builders_struct.service_type;
    match service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builders_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);

            match service_builder.open() {
                Ok(port_factory) => {
                    (*port_factory_struct_ptr).init(
                        service_type,
                        PortFactoryPubSubUnion::new_ipc(port_factory),
                        deleter,
                    );
                }
                Err(error) => {
                    return error.into_c_int();
                }
            }
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builders_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.pub_sub);

            match service_builder.open() {
                Ok(port_factory) => {
                    (*port_factory_struct_ptr).init(
                        service_type,
                        PortFactoryPubSubUnion::new_local(port_factory),
                        deleter,
                    );
                }
                Err(error) => {
                    return error.into_c_int();
                }
            }
        }
    }

    *port_factory_handle_ptr = (*port_factory_struct_ptr).as_handle();

    IOX2_OK
}

// END C API
