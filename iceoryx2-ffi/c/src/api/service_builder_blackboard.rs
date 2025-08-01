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

use super::{iox2_attribute_specifier_h_ref, iox2_attribute_verifier_h_ref, iox2_type_variant_e};
use crate::api::{
    c_size_t, iox2_port_factory_blackboard_h, iox2_port_factory_blackboard_t,
    iox2_service_builder_blackboard_creator_h, iox2_service_builder_blackboard_creator_h_ref,
    iox2_service_builder_blackboard_opener_h, iox2_service_builder_blackboard_opener_h_ref,
    iox2_service_type_e, AssertNonNullHandle, HandleToType, IntoCInt, KeyFfi,
    PortFactoryBlackboardUnion, ServiceBuilderUnion, IOX2_OK,
};
use crate::create_type_details;
use core::ffi::{c_char, c_int};
use core::mem::ManuallyDrop;
use iceoryx2::service::builder::blackboard::{
    BlackboardCreateError, BlackboardOpenError, Creator, Opener,
};
use iceoryx2::service::port_factory::blackboard::PortFactory;
use iceoryx2_bb_elementary_traits::AsCStr;
use iceoryx2_ffi_macros::CStrRepr;

// BEGIN types definition

#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_blackboard_open_or_create_error_e {
    #[CStr = "does not exist"]
    O_DOES_NOT_EXIST = IOX2_OK as isize + 1,
    #[CStr = "service in corrupted state"]
    O_SERVICE_IN_CORRUPTED_STATE,
    #[CStr = "incompatible keys"]
    O_INCOMPATIBLE_KEYS,
    #[CStr = "internal failure"]
    O_INTERNAL_FAILURE,
    #[CStr = "incompatible attributes"]
    O_INCOMPATIBLE_ATTRIBUTES,
    #[CStr = "incompatible messaging pattern"]
    O_INCOMPATIBLE_MESSAGING_PATTERN,
    #[CStr = "does not support requested amount of readers"]
    O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_READERS,
    #[CStr = "insufficient permissions"]
    O_INSUFFICIENT_PERMISSIONS,
    #[CStr = "hangs in creation"]
    O_HANGS_IN_CREATION,
    #[CStr = "is marked for destruction"]
    O_IS_MARKED_FOR_DESTRUCTION,
    #[CStr = "exceeds max number of nodes"]
    O_EXCEEDS_MAX_NUMBER_OF_NODES,
    #[CStr = "does not support requested amount of nodes"]
    O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NODES,
    #[CStr = "already exists"]
    C_ALREADY_EXISTS,
    #[CStr = "is being created by another instance"]
    C_IS_BEING_CREATED_BY_ANOTHER_INSTANCE,
    #[CStr = "internal failure"]
    C_INTERNAL_FAILURE,
    #[CStr = "insufficient permissions"]
    C_INSUFFICIENT_PERMISSIONS,
    #[CStr = "service in corrupted state"]
    C_SERVICE_IN_CORRUPTED_STATE,
    #[CStr = "hangs in creation"]
    C_HANGS_IN_CREATION,
    #[CStr = "no entries provided"]
    C_NO_ENTRIES_PROVIDED,
}

impl IntoCInt for BlackboardOpenError {
    fn into_c_int(self) -> c_int {
        (match self {
            BlackboardOpenError::DoesNotExist => {
                iox2_blackboard_open_or_create_error_e::O_DOES_NOT_EXIST
            }
            BlackboardOpenError::ServiceInCorruptedState => {
                iox2_blackboard_open_or_create_error_e::O_SERVICE_IN_CORRUPTED_STATE
            }
            BlackboardOpenError::IncompatibleKeys => {
                iox2_blackboard_open_or_create_error_e::O_INCOMPATIBLE_KEYS
            }
            BlackboardOpenError::InternalFailure => {
                iox2_blackboard_open_or_create_error_e::O_INTERNAL_FAILURE
            }
            BlackboardOpenError::IncompatibleAttributes => {
                iox2_blackboard_open_or_create_error_e::O_INCOMPATIBLE_ATTRIBUTES
            }
            BlackboardOpenError::IncompatibleMessagingPattern => {
                iox2_blackboard_open_or_create_error_e::O_INCOMPATIBLE_MESSAGING_PATTERN
            }
            BlackboardOpenError::DoesNotSupportRequestedAmountOfReaders => {
                iox2_blackboard_open_or_create_error_e::O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_READERS
            }
            BlackboardOpenError::InsufficientPermissions => {
                iox2_blackboard_open_or_create_error_e::O_INSUFFICIENT_PERMISSIONS
            }
            BlackboardOpenError::HangsInCreation => {
                iox2_blackboard_open_or_create_error_e::O_HANGS_IN_CREATION
            }
            BlackboardOpenError::IsMarkedForDestruction => {
                iox2_blackboard_open_or_create_error_e::O_IS_MARKED_FOR_DESTRUCTION
            }
            BlackboardOpenError::ExceedsMaxNumberOfNodes => {
                iox2_blackboard_open_or_create_error_e::O_EXCEEDS_MAX_NUMBER_OF_NODES
            }
            BlackboardOpenError::DoesNotSupportRequestedAmountOfNodes => {
                iox2_blackboard_open_or_create_error_e::O_DOES_NOT_SUPPORT_REQUESTED_AMOUNT_OF_NODES
            }
        }) as c_int
    }
}

impl IntoCInt for BlackboardCreateError {
    fn into_c_int(self) -> c_int {
        (match self {
            BlackboardCreateError::AlreadyExists => {
                iox2_blackboard_open_or_create_error_e::C_ALREADY_EXISTS
            }
            BlackboardCreateError::IsBeingCreatedByAnotherInstance => {
                iox2_blackboard_open_or_create_error_e::C_IS_BEING_CREATED_BY_ANOTHER_INSTANCE
            }
            BlackboardCreateError::InternalFailure => {
                iox2_blackboard_open_or_create_error_e::C_INTERNAL_FAILURE
            }
            BlackboardCreateError::InsufficientPermissions => {
                iox2_blackboard_open_or_create_error_e::C_INSUFFICIENT_PERMISSIONS
            }
            BlackboardCreateError::ServiceInCorruptedState => {
                iox2_blackboard_open_or_create_error_e::C_SERVICE_IN_CORRUPTED_STATE
            }
            BlackboardCreateError::HangsInCreation => {
                iox2_blackboard_open_or_create_error_e::C_HANGS_IN_CREATION
            }
            BlackboardCreateError::NoEntriesProvided => {
                iox2_blackboard_open_or_create_error_e::C_NO_ENTRIES_PROVIDED
            }
        }) as c_int
    }
}

// END type definition

// BEGIN C API

/// Returns a string literal describing the provided [`iox2_blackboard_open_or_create_error_e`].
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
pub unsafe extern "C" fn iox2_blackboard_open_or_create_error_string(
    error: iox2_blackboard_open_or_create_error_e,
) -> *const c_char {
    error.as_const_cstr().as_ptr() as *const c_char
}

/// Sets the key type details for the creator
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_blackboard_creator_h_ref`]
///   obtained by
///   [`iox2_service_builder_blackboard_creator`](crate::iox2_service_builder_blackboard).
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
pub unsafe extern "C" fn iox2_service_builder_blackboard_set_key_type_details(
    service_builder_handle: iox2_service_builder_blackboard_creator_h_ref,
    type_name_str: *const c_char,
    type_name_len: c_size_t,
    size: c_size_t,
    alignment: c_size_t,
) -> c_int {
    service_builder_handle.assert_non_null();

    let value = match create_type_details(
        iox2_type_variant_e::FIXED_SIZE,
        type_name_str,
        type_name_len,
        size,
        alignment,
    ) {
        Ok(v) => v,
        Err(e) => return e,
    };

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.blackboard_creator);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_blackboard_creator(
                service_builder.__internal_set_key_type_details(&value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.blackboard_creator);
            service_builder_struct.set(ServiceBuilderUnion::new_local_blackboard_creator(
                service_builder.__internal_set_key_type_details(&value),
            ));
        }
    }

    IOX2_OK
}

/// Sets the max readers for the creator
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_blackboard_creator_h_ref`]
///   obtained by
///   [`iox2_service_builder_blackboard_creator`](crate::iox2_service_builder_blackboard).
/// * `value` - The value to set the max readers to
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_blackboard_creator_set_max_readers(
    service_builder_handle: iox2_service_builder_blackboard_creator_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.blackboard_creator);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_blackboard_creator(
                service_builder.max_readers(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.blackboard_creator);
            service_builder_struct.set(ServiceBuilderUnion::new_local_blackboard_creator(
                service_builder.max_readers(value),
            ));
        }
    }
}

/// Sets the max readers for the opener
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_blackboard_opener_h_ref`]
///   obtained by
///   [`iox2_service_builder_blackboard_opener`](crate::iox2_service_builder_blackboard).
/// * `value` - The value to set the max readers to
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_blackboard_opener_set_max_readers(
    service_builder_handle: iox2_service_builder_blackboard_opener_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.blackboard_opener);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_blackboard_opener(
                service_builder.max_readers(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.blackboard_opener);
            service_builder_struct.set(ServiceBuilderUnion::new_local_blackboard_opener(
                service_builder.max_readers(value),
            ));
        }
    }
}

/// Sets the max nodes for the creator
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_blackboard_creator_h_ref`]
///   obtained by
///   [`iox2_service_builder_blackboard_creator`](crate::iox2_service_builder_blackboard).
/// * `value` - The value to set the max nodes to
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_blackboard_creator_set_max_nodes(
    service_builder_handle: iox2_service_builder_blackboard_creator_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.blackboard_creator);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_blackboard_creator(
                service_builder.max_nodes(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.blackboard_creator);
            service_builder_struct.set(ServiceBuilderUnion::new_local_blackboard_creator(
                service_builder.max_nodes(value),
            ));
        }
    }
}

/// Sets the max nodes for the opener
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_blackboard_opener_h_ref`]
///   obtained by
///   [`iox2_service_builder_blackboard_opener`](crate::iox2_service_builder_blackboard).
/// * `value` - The value to set the max nodes to
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_blackboard_opener_set_max_nodes(
    service_builder_handle: iox2_service_builder_blackboard_opener_h_ref,
    value: c_size_t,
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.blackboard_opener);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_blackboard_opener(
                service_builder.max_nodes(value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.blackboard_opener);
            service_builder_struct.set(ServiceBuilderUnion::new_local_blackboard_opener(
                service_builder.max_nodes(value),
            ));
        }
    }
}

// TODO: add + add_with_default

/// Adds key-value pairs to the blackboard.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_blackboard_creator_h_ref`]
///   obtained by
///   [`iox2_service_builder_blackboard_creator`](crate::iox2_service_builder_blackboard).
/// * `key` - The key that shall be added to the blackboard
/// * `value` - The value that shall be added to the blackboard
///
/// # Safety
///
/// * `service_builder_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_blackboard_creator_add(
    service_builder_handle: iox2_service_builder_blackboard_creator_h_ref,
    key: c_size_t,   // TODO: KeyType
    value: c_size_t, // TODO: ValueType
) {
    service_builder_handle.assert_non_null();

    let service_builder_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builder_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.blackboard_creator);
            service_builder_struct.set(ServiceBuilderUnion::new_ipc_blackboard_creator(
                service_builder.add(key, value),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builder_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.blackboard_creator);
            service_builder_struct.set(ServiceBuilderUnion::new_local_blackboard_creator(
                service_builder.add(key, value),
            ));
        }
    }
}

/// Opens an blackboard service and returns a port factory to create writers and readers.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_blackboard_opener_h`]
///   obtained by [`iox2_service_builder_blackboard_opener`](crate::iox2_service_builder_blackboard)
/// * `port_factory_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_port_factory_blackboard_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `port_factory_handle_ptr` - An uninitialized or dangling [`iox2_port_factory_blackboard_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_blackboard_open_or_create_error_e`] otherwise. Note, only the errors annotated with `O_` are relevant.
///
/// # Safety
///
/// * The `service_builder_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_service_builder_t`](crate::iox2_service_builder_t) can be re-used with
///   a call to [`iox2_node_service_builder`](crate::iox2_node_service_builder)!
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_blackboard_open(
    service_builder_handle: iox2_service_builder_blackboard_opener_h,
    port_factory_struct_ptr: *mut iox2_port_factory_blackboard_t,
    port_factory_handle_ptr: *mut iox2_port_factory_blackboard_h,
) -> c_int {
    iox2_service_builder_blackboard_open_impl(
        service_builder_handle,
        port_factory_struct_ptr,
        port_factory_handle_ptr,
        |service_builder| service_builder.open(),
        |service_builder| service_builder.open(),
    )
}

/// Opens an blackboard service and returns a port factory to create writers and readers.
/// The provided attributes are considered as requirements.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_blackboard_opener_h`]
///   obtained by [`iox2_service_builder_blackboard_opener`](crate::iox2_service_builder_blackboard)
/// * `port_factory_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_port_factory_blackboard_t`]). If it is a NULL pointer, the storage will be allocated on the heap.
/// * `port_factory_handle_ptr` - An uninitialized or dangling [`iox2_port_factory_blackboard_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_blackboard_open_or_create_error_e`] otherwise.
///
/// # Safety
///
/// * The `service_builder_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_service_builder_t`](crate::iox2_service_builder_t) can be re-used with
///   a call to [`iox2_node_service_builder`](crate::iox2_node_service_builder)!
/// * The `attribute_verifier_handle` must be valid.
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_blackboard_open_with_attributes(
    service_builder_handle: iox2_service_builder_blackboard_opener_h,
    attribute_verifier_handle: iox2_attribute_verifier_h_ref,
    port_factory_struct_ptr: *mut iox2_port_factory_blackboard_t,
    port_factory_handle_ptr: *mut iox2_port_factory_blackboard_h,
) -> c_int {
    let attribute_verifier_struct = &mut *attribute_verifier_handle.as_type();
    let attribute_verifier = &attribute_verifier_struct.value.as_ref().0;

    iox2_service_builder_blackboard_open_impl(
        service_builder_handle,
        port_factory_struct_ptr,
        port_factory_handle_ptr,
        |service_builder| service_builder.open_with_attributes(attribute_verifier),
        |service_builder| service_builder.open_with_attributes(attribute_verifier),
    )
}

/// Creates an blackbosrd service and returns a port factory to create writers and readers.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_blackboard_creator_h`](crate::iox2_service_builder_blackboard_creator_h)
///   obtained by
///   [`iox2_service_builder_blackboard_creator`](crate::iox2_service_builder_blackboard)
/// * `port_factory_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_port_factory_blackboard_t`](crate::iox2_port_factory_blackboard_t). If it is a NULL pointer, the storage will be allocated on the heap.
/// * `port_factory_handle_ptr` - An uninitialized or dangling [`iox2_port_factory_blackboard_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_blackboard_open_or_create_error_e`] otherwise. Note, only the errors annotated with `O_` are relevant.
///
/// # Safety
///
/// * The `service_builder_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_service_builder_t`](crate::iox2_service_builder_t) can be re-used with
///   a call to [`iox2_node_service_builder`](crate::iox2_node_service_builder)!
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_blackboard_create(
    service_builder_handle: iox2_service_builder_blackboard_creator_h,
    port_factory_struct_ptr: *mut iox2_port_factory_blackboard_t,
    port_factory_handle_ptr: *mut iox2_port_factory_blackboard_h,
) -> c_int {
    iox2_service_builder_blackboard_create_impl(
        service_builder_handle,
        port_factory_struct_ptr,
        port_factory_handle_ptr,
        |service_builder| service_builder.create(),
        |service_builder| service_builder.create(),
    )
}

/// Creates a service if it does not exist and returns a port factory to create writers and readers.
/// The provided arguments are stored inside the services.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_blackboard_creator_h`]
///   obtained by
///   [`iox2_service_builder_blackboard_creator`](crate::iox2_service_builder_blackboard)
/// * `port_factory_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///   [`iox2_port_factory_blackboard_t`]). If it is a NULL pointer, the storage will be allocated on the heap.
/// * `port_factory_handle_ptr` - An uninitialized or dangling [`iox2_port_factory_blackboard_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_blackboard_open_or_create_error_e`] otherwise.
///
/// # Safety
///
/// * The `service_builder_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_service_builder_t`](crate::iox2_service_builder_t) can be re-used with
///   a call to [`iox2_node_service_builder`](crate::iox2_node_service_builder)!
/// * The `attribute_verifier_handle` must be valid.
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_blackboard_create_with_attributes(
    service_builder_handle: iox2_service_builder_blackboard_creator_h,
    attribute_specifier_handle: iox2_attribute_specifier_h_ref,
    port_factory_struct_ptr: *mut iox2_port_factory_blackboard_t,
    port_factory_handle_ptr: *mut iox2_port_factory_blackboard_h,
) -> c_int {
    let attribute_specifier_struct = &mut *attribute_specifier_handle.as_type();
    let attribute_specifier = &attribute_specifier_struct.value.as_ref().0;

    iox2_service_builder_blackboard_create_impl(
        service_builder_handle,
        port_factory_struct_ptr,
        port_factory_handle_ptr,
        |service_builder| service_builder.create_with_attributes(attribute_specifier),
        |service_builder| service_builder.create_with_attributes(attribute_specifier),
    )
}

unsafe fn iox2_service_builder_blackboard_open_impl<E: IntoCInt>(
    service_builder_handle: iox2_service_builder_blackboard_opener_h,
    port_factory_struct_ptr: *mut iox2_port_factory_blackboard_t,
    port_factory_handle_ptr: *mut iox2_port_factory_blackboard_h,
    func_ipc: impl FnOnce(
        Opener<KeyFfi, crate::IpcService>,
    ) -> Result<PortFactory<crate::IpcService, KeyFfi>, E>,
    func_local: impl FnOnce(
        Opener<KeyFfi, crate::LocalService>,
    ) -> Result<PortFactory<crate::LocalService, KeyFfi>, E>,
) -> c_int {
    debug_assert!(!service_builder_handle.is_null());
    debug_assert!(!port_factory_handle_ptr.is_null());

    let init_port_factory_struct_ptr =
        |port_factory_struct_ptr: *mut iox2_port_factory_blackboard_t| {
            let mut port_factory_struct_ptr = port_factory_struct_ptr;
            fn no_op(_: *mut iox2_port_factory_blackboard_t) {}
            let mut deleter: fn(*mut iox2_port_factory_blackboard_t) = no_op;
            if port_factory_struct_ptr.is_null() {
                port_factory_struct_ptr = iox2_port_factory_blackboard_t::alloc();
                deleter = iox2_port_factory_blackboard_t::dealloc;
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
            panic!("Trying to use an invalid 'iox2_service_builder_blackboard_h'!");
        });
    (service_builder_struct.deleter)(service_builder_struct);

    match service_type {
        iox2_service_type_e::IPC => {
            let service_builder = ManuallyDrop::into_inner(service_builder.ipc);
            let service_builder = ManuallyDrop::into_inner(service_builder.blackboard_opener);

            match func_ipc(service_builder) {
                Ok(port_factory) => {
                    let (port_factory_struct_ptr, deleter) =
                        init_port_factory_struct_ptr(port_factory_struct_ptr);
                    (*port_factory_struct_ptr).init(
                        service_type,
                        PortFactoryBlackboardUnion::new_ipc(port_factory),
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
            let service_builder = ManuallyDrop::into_inner(service_builder.blackboard_opener);

            match func_local(service_builder) {
                Ok(port_factory) => {
                    let (port_factory_struct_ptr, deleter) =
                        init_port_factory_struct_ptr(port_factory_struct_ptr);
                    (*port_factory_struct_ptr).init(
                        service_type,
                        PortFactoryBlackboardUnion::new_local(port_factory),
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

unsafe fn iox2_service_builder_blackboard_create_impl<E: IntoCInt>(
    service_builder_handle: iox2_service_builder_blackboard_creator_h,
    port_factory_struct_ptr: *mut iox2_port_factory_blackboard_t,
    port_factory_handle_ptr: *mut iox2_port_factory_blackboard_h,
    func_ipc: impl FnOnce(
        Creator<KeyFfi, crate::IpcService>,
    ) -> Result<PortFactory<crate::IpcService, KeyFfi>, E>,
    func_local: impl FnOnce(
        Creator<KeyFfi, crate::LocalService>,
    ) -> Result<PortFactory<crate::LocalService, KeyFfi>, E>,
) -> c_int {
    debug_assert!(!service_builder_handle.is_null());
    debug_assert!(!port_factory_handle_ptr.is_null());

    let init_port_factory_struct_ptr =
        |port_factory_struct_ptr: *mut iox2_port_factory_blackboard_t| {
            let mut port_factory_struct_ptr = port_factory_struct_ptr;
            fn no_op(_: *mut iox2_port_factory_blackboard_t) {}
            let mut deleter: fn(*mut iox2_port_factory_blackboard_t) = no_op;
            if port_factory_struct_ptr.is_null() {
                port_factory_struct_ptr = iox2_port_factory_blackboard_t::alloc();
                deleter = iox2_port_factory_blackboard_t::dealloc;
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
            panic!("Trying to use an invalid 'iox2_service_builder_blackboard_h'!");
        });
    (service_builder_struct.deleter)(service_builder_struct);

    match service_type {
        iox2_service_type_e::IPC => {
            let service_builder = ManuallyDrop::into_inner(service_builder.ipc);
            let service_builder = ManuallyDrop::into_inner(service_builder.blackboard_creator);

            match func_ipc(service_builder) {
                Ok(port_factory) => {
                    let (port_factory_struct_ptr, deleter) =
                        init_port_factory_struct_ptr(port_factory_struct_ptr);
                    (*port_factory_struct_ptr).init(
                        service_type,
                        PortFactoryBlackboardUnion::new_ipc(port_factory),
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
            let service_builder = ManuallyDrop::into_inner(service_builder.blackboard_creator);

            match func_local(service_builder) {
                Ok(port_factory) => {
                    let (port_factory_struct_ptr, deleter) =
                        init_port_factory_struct_ptr(port_factory_struct_ptr);
                    (*port_factory_struct_ptr).init(
                        service_type,
                        PortFactoryBlackboardUnion::new_local(port_factory),
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

// TODO: check handles in documentation

// END C API
