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
    c_size_t, iox2_port_factory_event_h, iox2_port_factory_event_t, iox2_service_builder_event_h,
    iox2_service_builder_event_h_ref, iox2_service_type_e, AssertNonNullHandle, HandleToType,
    IntoCInt, PortFactoryEventUnion, ServiceBuilderUnion, IOX2_OK,
};

use iceoryx2::prelude::*;
use iceoryx2::service::builder::event::{
    Builder, EventCreateError, EventOpenError, EventOpenOrCreateError,
};
use iceoryx2::service::port_factory::event::PortFactory;
use iceoryx2_bb_elementary_traits::AsCStr;
use iceoryx2_ffi_macros::CStrRepr;

use core::ffi::{c_char, c_int};
use core::mem::ManuallyDrop;
use core::time::Duration;

use super::{iox2_attribute_specifier_h_ref, iox2_attribute_verifier_h_ref};

// BEGIN types definition

#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_event_open_or_create_error_e {
    #[CStr = "does not exist"]
    O_DOES_NOT_EXIST = IOX2_OK as isize + 1,
    #[CStr = "insufficient permissions"]
    O_INSUFFICIENT_PERMISSIONS,
    #[CStr = "service in corrupted state"]
    O_SERVICE_IN_CORRUPTED_STATE,
    #[CStr = "incompatible messaging pattern"]
    O_INCOMPATIBLE_MESSAGING_PATTERN,
    #[CStr = "incompatible attributes"]
    O_INCOMPATIBLE_ATTRIBUTES,
    #[CStr = "incompatible deadline"]
    O_INCOMPATIBLE_DEADLINE,
    #[CStr = "incompatible notifier_created event"]
    O_INCOMPATIBLE_NOTIFIER_CREATED_EVENT,
    #[CStr = "incompatible notifier_dropped event"]
    O_INCOMPATIBLE_NOTIFIER_DROPPED_EVENT,
    #[CStr = "incompatible notifier_dead event"]
    O_INCOMPATIBLE_NOTIFIER_DEAD_EVENT,
    #[CStr = "internal failure"]
    O_INTERNAL_FAILURE,
    #[CStr = "hangs in creation"]
    O_HANGS_IN_CREATION,
    #[CStr = "does not support requested amount of notifiers"]
    O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NOTIFIERS,
    #[CStr = "does not support requested amount of listeners"]
    O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_LISTENERS,
    #[CStr = "does not support requested max event id"]
    O_DOES_NOT_SUPPORT_REQUESTED_MAX_EVENT_ID,
    #[CStr = "does not support requested amount of nodes"]
    O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NODES,
    #[CStr = "exceeds max number of nodes"]
    O_EXCEEDS_MAX_NUMBER_OF_NODES,
    #[CStr = "is marked for destruction"]
    O_IS_MARKED_FOR_DESTRUCTION,
    #[CStr = "service in corrupted state"]
    C_SERVICE_IN_CORRUPTED_STATE,
    #[CStr = "internal failure"]
    C_INTERNAL_FAILURE,
    #[CStr = "is being created by another instance"]
    C_IS_BEING_CREATED_BY_ANOTHER_INSTANCE,
    #[CStr = "already exists"]
    C_ALREADY_EXISTS,
    #[CStr = "hangs in creation"]
    C_HANGS_IN_CREATION,
    #[CStr = "insufficient permissions"]
    C_INSUFFICIENT_PERMISSIONS,
    #[CStr = "old connection still active"]
    C_OLD_CONNECTION_STILL_ACTIVE,
    #[CStr = "same service is created and removed repeatedly"]
    SYSTEM_IN_FLUX,
}

impl IntoCInt for EventOpenError {
    fn into_c_int(self) -> c_int {
        (match self {
            EventOpenError::DoesNotExist => iox2_event_open_or_create_error_e::O_DOES_NOT_EXIST,
            EventOpenError::InsufficientPermissions => {
                iox2_event_open_or_create_error_e::O_INSUFFICIENT_PERMISSIONS
            }
            EventOpenError::ServiceInCorruptedState => {
                iox2_event_open_or_create_error_e::O_SERVICE_IN_CORRUPTED_STATE
            }
            EventOpenError::IncompatibleMessagingPattern => {
                iox2_event_open_or_create_error_e::O_INCOMPATIBLE_MESSAGING_PATTERN
            }
            EventOpenError::IncompatibleAttributes => {
                iox2_event_open_or_create_error_e::O_INCOMPATIBLE_ATTRIBUTES
            }
            EventOpenError::InternalFailure => {
                iox2_event_open_or_create_error_e::O_INTERNAL_FAILURE
            }
            EventOpenError::HangsInCreation => {
                iox2_event_open_or_create_error_e::O_HANGS_IN_CREATION
            }
            EventOpenError::DoesNotSupportRequestedAmountOfNotifiers => {
                iox2_event_open_or_create_error_e::O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NOTIFIERS
            }
            EventOpenError::DoesNotSupportRequestedAmountOfListeners => {
                iox2_event_open_or_create_error_e::O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_LISTENERS
            }
            EventOpenError::DoesNotSupportRequestedMaxEventId => {
                iox2_event_open_or_create_error_e::O_DOES_NOT_SUPPORT_REQUESTED_MAX_EVENT_ID
            }
            EventOpenError::DoesNotSupportRequestedAmountOfNodes => {
                iox2_event_open_or_create_error_e::O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NODES
            }
            EventOpenError::ExceedsMaxNumberOfNodes => {
                iox2_event_open_or_create_error_e::O_EXCEEDS_MAX_NUMBER_OF_NODES
            }
            EventOpenError::IsMarkedForDestruction => {
                iox2_event_open_or_create_error_e::O_IS_MARKED_FOR_DESTRUCTION
            }
            EventOpenError::IncompatibleNotifierCreatedEvent => {
                iox2_event_open_or_create_error_e::O_INCOMPATIBLE_NOTIFIER_CREATED_EVENT
            }
            EventOpenError::IncompatibleNotifierDroppedEvent => {
                iox2_event_open_or_create_error_e::O_INCOMPATIBLE_NOTIFIER_DROPPED_EVENT
            }
            EventOpenError::IncompatibleNotifierDeadEvent => {
                iox2_event_open_or_create_error_e::O_INCOMPATIBLE_NOTIFIER_DEAD_EVENT
            }
            EventOpenError::IncompatibleDeadline => {
                iox2_event_open_or_create_error_e::O_INCOMPATIBLE_DEADLINE
            }
        }) as c_int
    }
}

impl IntoCInt for EventCreateError {
    fn into_c_int(self) -> c_int {
        (match self {
            EventCreateError::ServiceInCorruptedState => {
                iox2_event_open_or_create_error_e::C_SERVICE_IN_CORRUPTED_STATE
            }

            EventCreateError::InternalFailure => {
                iox2_event_open_or_create_error_e::C_INTERNAL_FAILURE
            }
            EventCreateError::IsBeingCreatedByAnotherInstance => {
                iox2_event_open_or_create_error_e::C_IS_BEING_CREATED_BY_ANOTHER_INSTANCE
            }
            EventCreateError::AlreadyExists => iox2_event_open_or_create_error_e::C_ALREADY_EXISTS,
            EventCreateError::HangsInCreation => {
                iox2_event_open_or_create_error_e::C_HANGS_IN_CREATION
            }
            EventCreateError::InsufficientPermissions => {
                iox2_event_open_or_create_error_e::C_INSUFFICIENT_PERMISSIONS
            }
        }) as c_int
    }
}

impl IntoCInt for EventOpenOrCreateError {
    fn into_c_int(self) -> c_int {
        match self {
            EventOpenOrCreateError::EventOpenError(error) => error.into_c_int(),
            EventOpenOrCreateError::EventCreateError(error) => error.into_c_int(),
            e => e.into_c_int(),
        }
    }
}

// END type definition

// BEGIN C API

/// Returns a string literal describing the provided [`iox2_event_open_or_create_error_e`].
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
pub unsafe extern "C" fn iox2_event_open_or_create_error_string(
    error: iox2_event_open_or_create_error_e,
) -> *const c_char {
    error.as_const_cstr().as_ptr() as *const c_char
}

/// Enables the deadline property of the service. There must be a notification emitted by any
/// notifier after at least the provided `deadline`.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_event_h_ref`]
///   obtained by [`iox2_service_builder_event`](crate::iox2_service_builder_event).
/// * `seconds` - the second part of the deadline
/// * `nanoseconds` - the nanosecond part of the deadline
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_event_set_deadline(
    service_builder_handle: iox2_service_builder_event_h_ref,
    seconds: u64,
    nanoseconds: u32,
) {
    let deadline = Duration::from_secs(seconds) + Duration::from_nanos(nanoseconds as u64);
    iox2_service_builder_event_set_deadline_impl(service_builder_handle, Some(deadline));
}

/// Disables the deadline property of the service.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_event_h_ref`]
///   obtained by [`iox2_service_builder_event`](crate::iox2_service_builder_event).
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_event_disable_deadline(
    service_builder_handle: iox2_service_builder_event_h_ref,
) {
    iox2_service_builder_event_set_deadline_impl(service_builder_handle, None);
}

unsafe fn iox2_service_builder_event_set_deadline_impl(
    service_builder_handle: iox2_service_builder_event_h_ref,
    deadline: Option<Duration>,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };
    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.event);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_event(match deadline {
                Some(v) => service_builder.deadline(v),
                None => service_builder.disable_deadline(),
            }));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.event);
            service_builder_struct.set(ServiceBuilderUnion::new_local_event(match deadline {
                Some(v) => service_builder.deadline(v),
                None => service_builder.disable_deadline(),
            }));
        }
    }
}

/// Sets the event id value that shall be emitted if a notifier was identified as dead.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_event_h_ref`]
///   obtained by [`iox2_service_builder_event`](crate::iox2_service_builder_event).
/// * `value` - the value of the event id that will be emitted.
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_event_set_notifier_dead_event(
    service_builder_handle: iox2_service_builder_event_h_ref,
    value: c_size_t,
) {
    iox2_service_builder_event_set_notifier_dead_event_impl(
        service_builder_handle,
        Some(EventId::new(value as _)),
    );
}

/// Disables event id notification when a notifier was identified as dead.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_event_h_ref`]
///   obtained by [`iox2_service_builder_event`](crate::iox2_service_builder_event).
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_event_disable_notifier_dead_event(
    service_builder_handle: iox2_service_builder_event_h_ref,
) {
    iox2_service_builder_event_set_notifier_dead_event_impl(service_builder_handle, None);
}

#[no_mangle]
unsafe fn iox2_service_builder_event_set_notifier_dead_event_impl(
    service_builder_handle: iox2_service_builder_event_h_ref,
    value: Option<EventId>,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.event);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_event(match value {
                Some(value) => service_builder.notifier_dead_event(value),
                None => service_builder.disable_notifier_dead_event(),
            }));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.event);
            service_builder_struct.set(ServiceBuilderUnion::new_local_event(match value {
                Some(value) => service_builder.notifier_dead_event(value),
                None => service_builder.disable_notifier_dead_event(),
            }));
        }
    }
}

/// Sets the event id value that shall be emitted after a notifier was created.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_event_h_ref`]
///   obtained by [`iox2_service_builder_event`](crate::iox2_service_builder_event).
/// * `value` - the value of the event id that will be emitted.
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_event_set_notifier_created_event(
    service_builder_handle: iox2_service_builder_event_h_ref,
    value: c_size_t,
) {
    iox2_service_builder_event_set_notifier_created_event_impl(
        service_builder_handle,
        Some(EventId::new(value as _)),
    );
}

/// Disables the event id value that shall be emitted after a notifier was created.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_event_h_ref`]
///   obtained by [`iox2_service_builder_event`](crate::iox2_service_builder_event).
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_event_disable_notifier_created_event(
    service_builder_handle: iox2_service_builder_event_h_ref,
) {
    iox2_service_builder_event_set_notifier_created_event_impl(service_builder_handle, None);
}

unsafe fn iox2_service_builder_event_set_notifier_created_event_impl(
    service_builder_handle: iox2_service_builder_event_h_ref,
    value: Option<EventId>,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.event);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_event(match value {
                Some(value) => service_builder.notifier_created_event(value),
                None => service_builder.disable_notifier_created_event(),
            }));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.event);
            service_builder_struct.set(ServiceBuilderUnion::new_local_event(match value {
                Some(value) => service_builder.notifier_created_event(value),
                None => service_builder.disable_notifier_created_event(),
            }));
        }
    }
}

/// Sets the event id value that shall be emitted before a notifier is dropped.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_event_h_ref`]
///   obtained by [`iox2_service_builder_event`](crate::iox2_service_builder_event).
/// * `value` - the value of the event id that will be emitted.
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_event_set_notifier_dropped_event(
    service_builder_handle: iox2_service_builder_event_h_ref,
    value: c_size_t,
) {
    iox2_service_builder_event_set_notifier_dropped_event_impl(
        service_builder_handle,
        Some(EventId::new(value as _)),
    );
}

/// Disables the event id value that shall be emitted before a notifier is dropped.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_event_h_ref`]
///   obtained by [`iox2_service_builder_event`](crate::iox2_service_builder_event).
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_event_disable_notifier_dropped_event(
    service_builder_handle: iox2_service_builder_event_h_ref,
) {
    iox2_service_builder_event_set_notifier_dropped_event_impl(service_builder_handle, None);
}

#[no_mangle]
unsafe fn iox2_service_builder_event_set_notifier_dropped_event_impl(
    service_builder_handle: iox2_service_builder_event_h_ref,
    value: Option<EventId>,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.event);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_event(match value {
                Some(value) => service_builder.notifier_dropped_event(value),
                None => service_builder.disable_notifier_dropped_event(),
            }));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.event);
            service_builder_struct.set(ServiceBuilderUnion::new_local_event(match value {
                Some(value) => service_builder.notifier_dropped_event(value),
                None => service_builder.disable_notifier_dropped_event(),
            }));
        }
    }
}

/// Sets the max notifiers for the builder
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_event_h_ref`]
///   obtained by [`iox2_service_builder_event`](crate::iox2_service_builder_event).
/// * `value` - The value to set the max notifiers to
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_event_set_max_notifiers(
    service_builder_handle: iox2_service_builder_event_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.event);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_event(
                service_builder.max_notifiers(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.event);
            service_builder_struct.set(ServiceBuilderUnion::new_local_event(
                service_builder.max_notifiers(value),
            ));
        }
    }
}

/// Sets the max nodes for the builder
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_event_h_ref`]
///   obtained by [`iox2_service_builder_event`](crate::iox2_service_builder_event).
/// * `value` - The value to set the max notifiers to
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_event_set_max_nodes(
    service_builder_handle: iox2_service_builder_event_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.event);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_event(
                service_builder.max_nodes(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.event);
            service_builder_struct.set(ServiceBuilderUnion::new_local_event(
                service_builder.max_nodes(value),
            ));
        }
    }
}

/// Sets the max event id value for the builder
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_event_h_ref`]
///   obtained by [`iox2_service_builder_event`](crate::iox2_service_builder_event).
/// * `value` - The value to set the max notifiers to
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_event_set_event_id_max_value(
    service_builder_handle: iox2_service_builder_event_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.event);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_event(
                service_builder.event_id_max_value(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.event);
            service_builder_struct.set(ServiceBuilderUnion::new_local_event(
                service_builder.event_id_max_value(value),
            ));
        }
    }
}

/// Sets the max listeners for the builder
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_event_h_ref`]
///   obtained by [`iox2_service_builder_event`](crate::iox2_service_builder_event).
/// * `value` - The value to set the max listeners to
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_event_set_max_listeners(
    service_builder_handle: iox2_service_builder_event_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.event);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_event(
                service_builder.max_listeners(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.event);
            service_builder_struct.set(ServiceBuilderUnion::new_local_event(
                service_builder.max_listeners(value),
            ));
        }
    }
}

/// Opens an event service or creates the service if it does not exist and returns a port factory to create notifiers and listeners.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_event_h`]
///   obtained by [`iox2_service_builder_event`](crate::iox2_service_builder_event)
/// * `port_factory_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_port_factory_event_t`]). If it is a NULL pointer, the storage will be allocated on the heap.
/// * `port_factory_handle_ptr` - An uninitialized or dangling [`iox2_port_factory_event_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_event_open_or_create_error_e`] otherwise.
///
/// # Safety
///
/// * The `service_builder_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_service_builder_t`](crate::iox2_service_builder_t) can be re-used with
///   a call to [`iox2_node_service_builder`](crate::iox2_node_service_builder)!
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_event_open_or_create(
    service_builder_handle: iox2_service_builder_event_h,
    port_factory_struct_ptr: *mut iox2_port_factory_event_t,
    port_factory_handle_ptr: *mut iox2_port_factory_event_h,
) -> c_int {
    iox2_service_builder_event_open_create_impl(
        service_builder_handle,
        port_factory_struct_ptr,
        port_factory_handle_ptr,
        |service_builder| service_builder.open_or_create(),
        |service_builder| service_builder.open_or_create(),
    )
}

/// Opens an event service or creates the service if it does not exist and returns a port factory to create notifiers and listeners.
/// If the service does not exist, the provided arguments are stored inside the services, if the
/// service already exists, the provided attributes are considered as requirements.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_event_h`]
///   obtained by [`iox2_service_builder_event`](crate::iox2_service_builder_event)
/// * `port_factory_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_port_factory_event_t`]). If it is a NULL pointer, the storage will be allocated on the heap.
/// * `port_factory_handle_ptr` - An uninitialized or dangling [`iox2_port_factory_event_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_event_open_or_create_error_e`] otherwise.
///
/// # Safety
///
/// * The `service_builder_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_service_builder_t`](crate::iox2_service_builder_t) can be re-used with
///   a call to [`iox2_node_service_builder`](crate::iox2_node_service_builder)!
/// * The `attribute_verifier_handle` must be valid.
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_event_open_or_create_with_attributes(
    service_builder_handle: iox2_service_builder_event_h,
    attribute_verifier_handle: iox2_attribute_verifier_h_ref,
    port_factory_struct_ptr: *mut iox2_port_factory_event_t,
    port_factory_handle_ptr: *mut iox2_port_factory_event_h,
) -> c_int {
    let attribute_verifier_struct = &mut *attribute_verifier_handle.as_type();
    let attribute_verifier = &attribute_verifier_struct.value.as_ref().0;

    iox2_service_builder_event_open_create_impl(
        service_builder_handle,
        port_factory_struct_ptr,
        port_factory_handle_ptr,
        |service_builder| service_builder.open_or_create_with_attributes(attribute_verifier),
        |service_builder| service_builder.open_or_create_with_attributes(attribute_verifier),
    )
}

/// Opens an event service and returns a port factory to create notifiers and listeners.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_event_h`]
///   obtained by [`iox2_service_builder_event`](crate::iox2_service_builder_event)
/// * `port_factory_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_port_factory_event_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `port_factory_handle_ptr` - An uninitialized or dangling [`iox2_port_factory_event_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_event_open_or_create_error_e`] otherwise. Note, only the errors annotated with `O_` are relevant.
///
/// # Safety
///
/// * The `service_builder_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_service_builder_t`](crate::iox2_service_builder_t) can be re-used with
///   a call to [`iox2_node_service_builder`](crate::iox2_node_service_builder)!
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_event_open(
    service_builder_handle: iox2_service_builder_event_h,
    port_factory_struct_ptr: *mut iox2_port_factory_event_t,
    port_factory_handle_ptr: *mut iox2_port_factory_event_h,
) -> c_int {
    iox2_service_builder_event_open_create_impl(
        service_builder_handle,
        port_factory_struct_ptr,
        port_factory_handle_ptr,
        |service_builder| service_builder.open(),
        |service_builder| service_builder.open(),
    )
}

/// Opens an event service and returns a port factory to create notifiers and listeners.
/// The provided attributes are considered as requirements.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_event_h`]
///   obtained by [`iox2_service_builder_event`](crate::iox2_service_builder_event)
/// * `port_factory_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_port_factory_event_t`]). If it is a NULL pointer, the storage will be allocated on the heap.
/// * `port_factory_handle_ptr` - An uninitialized or dangling [`iox2_port_factory_event_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_event_open_or_create_error_e`] otherwise.
///
/// # Safety
///
/// * The `service_builder_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_service_builder_t`](crate::iox2_service_builder_t) can be re-used with
///   a call to [`iox2_node_service_builder`](crate::iox2_node_service_builder)!
/// * The `attribute_verifier_handle` must be valid.
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_event_open_with_attributes(
    service_builder_handle: iox2_service_builder_event_h,
    attribute_verifier_handle: iox2_attribute_verifier_h_ref,
    port_factory_struct_ptr: *mut iox2_port_factory_event_t,
    port_factory_handle_ptr: *mut iox2_port_factory_event_h,
) -> c_int {
    let attribute_verifier_struct = &mut *attribute_verifier_handle.as_type();
    let attribute_verifier = &attribute_verifier_struct.value.as_ref().0;

    iox2_service_builder_event_open_create_impl(
        service_builder_handle,
        port_factory_struct_ptr,
        port_factory_handle_ptr,
        |service_builder| service_builder.open_with_attributes(attribute_verifier),
        |service_builder| service_builder.open_with_attributes(attribute_verifier),
    )
}

/// Creates an event service and returns a port factory to create notifiers and listeners.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_event_h`](crate::iox2_service_builder_event_h)
///   obtained by [`iox2_service_builder_event`](crate::iox2_service_builder_event)
/// * `port_factory_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_port_factory_event_t`](crate::iox2_port_factory_event_t). If it is a NULL pointer, the storage will be allocated on the heap.
/// * `port_factory_handle_ptr` - An uninitialized or dangling [`iox2_port_factory_event_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_event_open_or_create_error_e`] otherwise. Note, only the errors annotated with `O_` are relevant.
///
/// # Safety
///
/// * The `service_builder_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_service_builder_t`](crate::iox2_service_builder_t) can be re-used with
///   a call to [`iox2_node_service_builder`](crate::iox2_node_service_builder)!
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_event_create(
    service_builder_handle: iox2_service_builder_event_h,
    port_factory_struct_ptr: *mut iox2_port_factory_event_t,
    port_factory_handle_ptr: *mut iox2_port_factory_event_h,
) -> c_int {
    iox2_service_builder_event_open_create_impl(
        service_builder_handle,
        port_factory_struct_ptr,
        port_factory_handle_ptr,
        |service_builder| service_builder.create(),
        |service_builder| service_builder.create(),
    )
}

/// Creates a service if it does not exist and returns a port factory to create notifiers and listeners.
/// The provided arguments are stored inside the services.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_event_h`]
///   obtained by [`iox2_service_builder_event`](crate::iox2_service_builder_event)
/// * `port_factory_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_port_factory_event_t`]). If it is a NULL pointer, the storage will be allocated on the heap.
/// * `port_factory_handle_ptr` - An uninitialized or dangling [`iox2_port_factory_event_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_event_open_or_create_error_e`] otherwise.
///
/// # Safety
///
/// * The `service_builder_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_service_builder_t`](crate::iox2_service_builder_t) can be re-used with
///   a call to [`iox2_node_service_builder`](crate::iox2_node_service_builder)!
/// * The `attribute_verifier_handle` must be valid.
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_event_create_with_attributes(
    service_builder_handle: iox2_service_builder_event_h,
    attribute_specifier_handle: iox2_attribute_specifier_h_ref,
    port_factory_struct_ptr: *mut iox2_port_factory_event_t,
    port_factory_handle_ptr: *mut iox2_port_factory_event_h,
) -> c_int {
    let attribute_specifier_struct = &mut *attribute_specifier_handle.as_type();
    let attribute_specifier = &attribute_specifier_struct.value.as_ref().0;

    iox2_service_builder_event_open_create_impl(
        service_builder_handle,
        port_factory_struct_ptr,
        port_factory_handle_ptr,
        |service_builder| service_builder.create_with_attributes(attribute_specifier),
        |service_builder| service_builder.create_with_attributes(attribute_specifier),
    )
}

unsafe fn iox2_service_builder_event_open_create_impl<E: IntoCInt>(
    service_builder_handle: iox2_service_builder_event_h,
    port_factory_struct_ptr: *mut iox2_port_factory_event_t,
    port_factory_handle_ptr: *mut iox2_port_factory_event_h,
    func_ipc: impl FnOnce(Builder<crate::IpcService>) -> Result<PortFactory<crate::IpcService>, E>,
    func_local: impl FnOnce(Builder<crate::LocalService>) -> Result<PortFactory<crate::LocalService>, E>,
) -> c_int {
    debug_assert!(!service_builder_handle.is_null());
    debug_assert!(!port_factory_handle_ptr.is_null());

    let init_port_factory_struct_ptr = |port_factory_struct_ptr: *mut iox2_port_factory_event_t| {
        let mut port_factory_struct_ptr = port_factory_struct_ptr;
        fn no_op(_: *mut iox2_port_factory_event_t) {}
        let mut deleter: fn(*mut iox2_port_factory_event_t) = no_op;
        if port_factory_struct_ptr.is_null() {
            port_factory_struct_ptr = iox2_port_factory_event_t::alloc();
            deleter = iox2_port_factory_event_t::dealloc;
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
            panic!("Trying to use an invalid 'iox2_service_builder_event_h'!");
        });
    (service_builder_struct.deleter)(service_builder_struct);

    match service_type {
        iox2_service_type_e::IPC => {
            let service_builder = ManuallyDrop::into_inner(service_builder.ipc);
            let service_builder = ManuallyDrop::into_inner(service_builder.event);

            match func_ipc(service_builder) {
                Ok(port_factory) => {
                    let (port_factory_struct_ptr, deleter) =
                        init_port_factory_struct_ptr(port_factory_struct_ptr);
                    (*port_factory_struct_ptr).init(
                        service_type,
                        PortFactoryEventUnion::new_ipc(port_factory),
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
            let service_builder = ManuallyDrop::into_inner(service_builder.event);

            match func_local(service_builder) {
                Ok(port_factory) => {
                    let (port_factory_struct_ptr, deleter) =
                        init_port_factory_struct_ptr(port_factory_struct_ptr);
                    (*port_factory_struct_ptr).init(
                        service_type,
                        PortFactoryEventUnion::new_local(port_factory),
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
