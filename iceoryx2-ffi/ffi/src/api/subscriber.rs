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
    c_size_t, iox2_sample_h, iox2_sample_t, iox2_service_type_e, iox2_unique_subscriber_id_h,
    iox2_unique_subscriber_id_t, AssertNonNullHandle, HandleToType, IntoCInt, PayloadFfi,
    SampleUnion, UserHeaderFfi, IOX2_OK,
};

use iceoryx2::port::subscriber::{Subscriber, SubscriberReceiveError};
use iceoryx2::port::update_connections::{ConnectionFailure, UpdateConnections};
use iceoryx2::prelude::*;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::ffi::c_int;
use core::mem::ManuallyDrop;

// BEGIN types definition

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_subscriber_receive_error_e {
    EXCEEDS_MAX_BORROWED_SAMPLES = IOX2_OK as isize + 1,
    FAILED_TO_ESTABLISH_CONNECTION,
    UNABLE_TO_MAP_PUBLISHERS_DATA_SEGMENT,
}

impl IntoCInt for SubscriberReceiveError {
    fn into_c_int(self) -> c_int {
        (match self {
            SubscriberReceiveError::ExceedsMaxBorrowedSamples => {
                iox2_subscriber_receive_error_e::EXCEEDS_MAX_BORROWED_SAMPLES
            }
            SubscriberReceiveError::ConnectionFailure(
                ConnectionFailure::FailedToEstablishConnection(_),
            ) => iox2_subscriber_receive_error_e::FAILED_TO_ESTABLISH_CONNECTION,
            SubscriberReceiveError::ConnectionFailure(
                ConnectionFailure::UnableToMapPublishersDataSegment(_),
            ) => iox2_subscriber_receive_error_e::UNABLE_TO_MAP_PUBLISHERS_DATA_SEGMENT,
        }) as c_int
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_connection_failure_e {
    FAILED_TO_ESTABLISH_CONNECTION,
    UNABLE_TO_MAP_PUBLISHERS_DATA_SEGMENT,
}

impl IntoCInt for ConnectionFailure {
    fn into_c_int(self) -> c_int {
        (match self {
            ConnectionFailure::FailedToEstablishConnection(_) => {
                iox2_connection_failure_e::FAILED_TO_ESTABLISH_CONNECTION
            }
            ConnectionFailure::UnableToMapPublishersDataSegment(_) => {
                iox2_connection_failure_e::UNABLE_TO_MAP_PUBLISHERS_DATA_SEGMENT
            }
        }) as c_int
    }
}

pub(super) union SubscriberUnion {
    ipc: ManuallyDrop<Subscriber<ipc::Service, PayloadFfi, UserHeaderFfi>>,
    local: ManuallyDrop<Subscriber<local::Service, PayloadFfi, UserHeaderFfi>>,
}

impl SubscriberUnion {
    pub(super) fn new_ipc(subscriber: Subscriber<ipc::Service, PayloadFfi, UserHeaderFfi>) -> Self {
        Self {
            ipc: ManuallyDrop::new(subscriber),
        }
    }
    pub(super) fn new_local(
        subscriber: Subscriber<local::Service, PayloadFfi, UserHeaderFfi>,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(subscriber),
        }
    }
}

#[repr(C)]
#[repr(align(16))] // alignment of Option<SubscriberUnion>
pub struct iox2_subscriber_storage_t {
    internal: [u8; 816], // magic number obtained with size_of::<Option<SubscriberUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(SubscriberUnion)]
pub struct iox2_subscriber_t {
    service_type: iox2_service_type_e,
    value: iox2_subscriber_storage_t,
    deleter: fn(*mut iox2_subscriber_t),
}

impl iox2_subscriber_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: SubscriberUnion,
        deleter: fn(*mut iox2_subscriber_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_subscriber_h_t;
/// The owning handle for `iox2_subscriber_t`. Passing the handle to an function transfers the ownership.
pub type iox2_subscriber_h = *mut iox2_subscriber_h_t;
/// The non-owning handle for `iox2_subscriber_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_subscriber_h_ref = *const iox2_subscriber_h;

impl AssertNonNullHandle for iox2_subscriber_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_subscriber_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_subscriber_h {
    type Target = *mut iox2_subscriber_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_subscriber_h_ref {
    type Target = *mut iox2_subscriber_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition

// BEGIN C API

/// Returns the buffer size of the subscriber
///
/// # Arguments
///
/// * `subscriber_handle` - Must be a valid [`iox2_subscriber_h_ref`]
///   obtained by [`iox2_port_factory_subscriber_builder_create`](crate::iox2_port_factory_subscriber_builder_create).
///
/// # Safety
///
/// * `subscriber_handle` must be valid handles
#[no_mangle]
pub unsafe extern "C" fn iox2_subscriber_buffer_size(
    subscriber_handle: iox2_subscriber_h_ref,
) -> c_size_t {
    subscriber_handle.assert_non_null();

    let subscriber = &mut *subscriber_handle.as_type();

    match subscriber.service_type {
        iox2_service_type_e::IPC => subscriber.value.as_ref().ipc.buffer_size(),
        iox2_service_type_e::LOCAL => subscriber.value.as_ref().local.buffer_size(),
    }
}

/// Returns the unique port id of the subscriber.
///
/// # Arguments
///
/// * `subscriber_handle` obtained by [`iox2_port_factory_subscriber_builder_create`](crate::iox2_port_factory_subscriber_builder_create)
/// * `id_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_unique_subscriber_id_t`].
///                         If it is a NULL pointer, the storage will be allocated on the heap.
/// * `id_handle_ptr` valid pointer to a [`iox2_unique_subscriber_id_h`].
///
/// # Safety
///
/// * `subscriber_handle` is valid, non-null and was obtained via [`iox2_port_factory_subscriber_builder_create`](crate::iox2_port_factory_subscriber_builder_create)
/// * `id` is valid and non-null
#[no_mangle]
pub unsafe extern "C" fn iox2_subscriber_id(
    subscriber_handle: iox2_subscriber_h_ref,
    id_struct_ptr: *mut iox2_unique_subscriber_id_t,
    id_handle_ptr: *mut iox2_unique_subscriber_id_h,
) {
    subscriber_handle.assert_non_null();
    debug_assert!(!id_handle_ptr.is_null());

    fn no_op(_: *mut iox2_unique_subscriber_id_t) {}
    let mut deleter: fn(*mut iox2_unique_subscriber_id_t) = no_op;
    let mut storage_ptr = id_struct_ptr;
    if id_struct_ptr.is_null() {
        deleter = iox2_unique_subscriber_id_t::dealloc;
        storage_ptr = iox2_unique_subscriber_id_t::alloc();
    }
    debug_assert!(!storage_ptr.is_null());

    let subscriber = &mut *subscriber_handle.as_type();

    let id = match subscriber.service_type {
        iox2_service_type_e::IPC => subscriber.value.as_mut().ipc.id(),
        iox2_service_type_e::LOCAL => subscriber.value.as_mut().local.id(),
    };

    (*storage_ptr).init(id, deleter);
    *id_handle_ptr = (*storage_ptr).as_handle();
}

// TODO [#210] add all the other setter methods

/// Takes a sample ouf of the subscriber queue.
///
/// # Arguments
///
/// * `subscriber_handle` - Must be a valid [`iox2_subscriber_h_ref`]
///   obtained by [`iox2_port_factory_subscriber_builder_create`](crate::iox2_port_factory_subscriber_builder_create).
/// * `sample_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_sample_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
/// * `sample_handle_ptr` - An uninitialized or dangling [`iox2_sample_h`] handle which will be initialized by this function call if a sample is obtained, otherwise it will be set to NULL.
///
/// Returns IOX2_OK on success, an [`iox2_subscriber_receive_error_e`] otherwise.
/// Attention, an empty subscriber queue is not an error and even with IOX2_OK it is possible to get a NULL in `sample_handle_ptr`.
///
/// # Safety
///
/// * The `subscriber_handle` is still valid after the return of this function and can be use in another function call.
/// * The `sample_handle_ptr` is pointing to a valid [`iox2_sample_h`].
#[no_mangle]
pub unsafe extern "C" fn iox2_subscriber_receive(
    subscriber_handle: iox2_subscriber_h_ref,
    sample_struct_ptr: *mut iox2_sample_t,
    sample_handle_ptr: *mut iox2_sample_h,
) -> c_int {
    subscriber_handle.assert_non_null();
    debug_assert!(!sample_handle_ptr.is_null());

    *sample_handle_ptr = std::ptr::null_mut();

    let init_sample_struct_ptr = |sample_struct_ptr: *mut iox2_sample_t| {
        let mut sample_struct_ptr = sample_struct_ptr;
        fn no_op(_: *mut iox2_sample_t) {}
        let mut deleter: fn(*mut iox2_sample_t) = no_op;
        if sample_struct_ptr.is_null() {
            sample_struct_ptr = iox2_sample_t::alloc();
            deleter = iox2_sample_t::dealloc;
        }
        debug_assert!(!sample_struct_ptr.is_null());

        (sample_struct_ptr, deleter)
    };

    let subscriber = &mut *subscriber_handle.as_type();

    match subscriber.service_type {
        iox2_service_type_e::IPC => match subscriber.value.as_ref().ipc.receive_custom_payload() {
            Ok(Some(sample)) => {
                let (sample_struct_ptr, deleter) = init_sample_struct_ptr(sample_struct_ptr);
                (*sample_struct_ptr).init(
                    subscriber.service_type,
                    SampleUnion::new_ipc(sample),
                    deleter,
                );
                *sample_handle_ptr = (*sample_struct_ptr).as_handle();
            }
            Ok(None) => (),
            Err(error) => return error.into_c_int(),
        },
        iox2_service_type_e::LOCAL => {
            match subscriber.value.as_ref().local.receive_custom_payload() {
                Ok(Some(sample)) => {
                    let (sample_struct_ptr, deleter) = init_sample_struct_ptr(sample_struct_ptr);
                    (*sample_struct_ptr).init(
                        subscriber.service_type,
                        SampleUnion::new_local(sample),
                        deleter,
                    );
                    *sample_handle_ptr = (*sample_struct_ptr).as_handle();
                }
                Ok(None) => (),
                Err(error) => return error.into_c_int(),
            }
        }
    }

    IOX2_OK
}

/// Returns true when the subscriber has samples that can be acquired with [`iox2_subscriber_receive`], otherwise false.
///
/// # Arguments
///
/// * `subscriber_handle` - Must be a valid [`iox2_subscriber_h_ref`]
///   obtained by [`iox2_port_factory_subscriber_builder_create`](crate::iox2_port_factory_subscriber_builder_create).
/// * `result_ptr` - A non-null pointer to a bool that will contain the result.
///
/// Returns IOX2_OK on success, an [`iox2_connection_failure_e`] otherwise.
/// Attention, an empty subscriber queue is not an error and even with IOX2_OK it is possible to get a NULL in `sample_handle_ptr`.
///
/// # Safety
///
/// * The `subscriber_handle` is still valid after the return of this function and can be use in another function call.
/// * The `result_ptr` is pointing to a valid bool.
#[no_mangle]
pub unsafe extern "C" fn iox2_subscriber_has_samples(
    subscriber_handle: iox2_subscriber_h_ref,
    result_ptr: *mut bool,
) -> c_int {
    subscriber_handle.assert_non_null();
    debug_assert!(!result_ptr.is_null());

    let subscriber = &mut *subscriber_handle.as_type();

    match subscriber.service_type {
        iox2_service_type_e::IPC => match subscriber.value.as_ref().ipc.has_samples() {
            Ok(v) => {
                *result_ptr = v;
                IOX2_OK
            }
            Err(error) => error.into_c_int(),
        },
        iox2_service_type_e::LOCAL => match subscriber.value.as_ref().local.has_samples() {
            Ok(v) => {
                *result_ptr = v;
                IOX2_OK
            }
            Err(error) => error.into_c_int(),
        },
    }
}

/// Updates all connections to new and obsolete publisher ports
///
/// # Arguments
///
/// * `subscriber_handle` - Must be a valid [`iox2_subscriber_h_ref`]
///   obtained by [`iox2_port_factory_subscriber_builder_create`](crate::iox2_port_factory_subscriber_builder_create).
///
/// # Safety
///
/// * The `subscriber_handle` is still valid after the return of this function and can be use in another function call.
#[no_mangle]
pub unsafe extern "C" fn iox2_subscriber_update_connections(
    subscriber_handle: iox2_subscriber_h_ref,
) -> c_int {
    subscriber_handle.assert_non_null();

    let subscriber = &mut *subscriber_handle.as_type();

    match subscriber.service_type {
        iox2_service_type_e::IPC => match subscriber.value.as_ref().ipc.update_connections() {
            Ok(()) => IOX2_OK,
            Err(error) => error.into_c_int(),
        },
        iox2_service_type_e::LOCAL => match subscriber.value.as_ref().local.update_connections() {
            Ok(()) => IOX2_OK,
            Err(error) => error.into_c_int(),
        },
    }
}

/// This function needs to be called to destroy the subscriber!
///
/// # Arguments
///
/// * `subscriber_handle` - A valid [`iox2_subscriber_h`]
///
/// # Safety
///
/// * The `subscriber_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_subscriber_t`] can be re-used with a call to
///   [`iox2_port_factory_subscriber_builder_create`](crate::iox2_port_factory_subscriber_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_subscriber_drop(subscriber_handle: iox2_subscriber_h) {
    subscriber_handle.assert_non_null();

    let subscriber = &mut *subscriber_handle.as_type();

    match subscriber.service_type {
        iox2_service_type_e::IPC => {
            ManuallyDrop::drop(&mut subscriber.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            ManuallyDrop::drop(&mut subscriber.value.as_mut().local);
        }
    }
    (subscriber.deleter)(subscriber);
}

// END C API
