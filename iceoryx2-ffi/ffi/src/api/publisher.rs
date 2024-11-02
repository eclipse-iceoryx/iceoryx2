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
    iox2_service_type_e, iox2_unable_to_deliver_strategy_e, iox2_unique_publisher_id_h,
    iox2_unique_publisher_id_t, AssertNonNullHandle, HandleToType, PayloadFfi,
    SampleMutUninitUnion, UserHeaderFfi, IOX2_OK,
};

use iceoryx2::port::publisher::{Publisher, PublisherLoanError, PublisherSendError};
use iceoryx2::port::update_connections::UpdateConnections;
use iceoryx2::prelude::*;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use super::{iox2_sample_mut_h, iox2_sample_mut_t, IntoCInt};

use core::ffi::{c_int, c_void};
use core::mem::ManuallyDrop;

// BEGIN types definition

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_publisher_send_error_e {
    CONNECTION_BROKEN_SINCE_PUBLISHER_NO_LONGER_EXISTS = IOX2_OK as isize + 1,
    CONNECTION_CORRUPTED,
    LOAN_ERROR_OUT_OF_MEMORY,
    LOAN_ERROR_EXCEEDS_MAX_LOANED_SAMPLES,
    LOAN_ERROR_EXCEEDS_MAX_LOAN_SIZE,
    LOAN_ERROR_INTERNAL_FAILURE,
    CONNECTION_ERROR,
}

impl IntoCInt for PublisherSendError {
    fn into_c_int(self) -> c_int {
        (match self {
            PublisherSendError::ConnectionBrokenSincePublisherNoLongerExists => {
                iox2_publisher_send_error_e::CONNECTION_BROKEN_SINCE_PUBLISHER_NO_LONGER_EXISTS
            }
            PublisherSendError::ConnectionCorrupted => {
                iox2_publisher_send_error_e::CONNECTION_CORRUPTED
            }
            PublisherSendError::LoanError(PublisherLoanError::OutOfMemory) => {
                iox2_publisher_send_error_e::LOAN_ERROR_OUT_OF_MEMORY
            }
            PublisherSendError::LoanError(PublisherLoanError::ExceedsMaxLoanedSamples) => {
                iox2_publisher_send_error_e::LOAN_ERROR_EXCEEDS_MAX_LOANED_SAMPLES
            }
            PublisherSendError::LoanError(PublisherLoanError::ExceedsMaxLoanSize) => {
                iox2_publisher_send_error_e::LOAN_ERROR_EXCEEDS_MAX_LOAN_SIZE
            }
            PublisherSendError::LoanError(PublisherLoanError::InternalFailure) => {
                iox2_publisher_send_error_e::LOAN_ERROR_INTERNAL_FAILURE
            }
            PublisherSendError::ConnectionError(_) => iox2_publisher_send_error_e::CONNECTION_ERROR,
        }) as c_int
    }
}

impl IntoCInt for PublisherLoanError {
    fn into_c_int(self) -> c_int {
        (match self {
            PublisherLoanError::OutOfMemory => iox2_publisher_loan_error_e::OUT_OF_MEMORY,
            PublisherLoanError::ExceedsMaxLoanedSamples => {
                iox2_publisher_loan_error_e::EXCEEDS_MAX_LOANED_SAMPLES
            }
            PublisherLoanError::ExceedsMaxLoanSize => {
                iox2_publisher_loan_error_e::EXCEEDS_MAX_LOAN_SIZE
            }
            PublisherLoanError::InternalFailure => iox2_publisher_loan_error_e::INTERNAL_FAILURE,
        }) as c_int
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_publisher_loan_error_e {
    OUT_OF_MEMORY = IOX2_OK as isize + 1,
    EXCEEDS_MAX_LOANED_SAMPLES,
    EXCEEDS_MAX_LOAN_SIZE,
    INTERNAL_FAILURE,
}

pub(super) union PublisherUnion {
    ipc: ManuallyDrop<Publisher<ipc::Service, PayloadFfi, UserHeaderFfi>>,
    local: ManuallyDrop<Publisher<local::Service, PayloadFfi, UserHeaderFfi>>,
}

impl PublisherUnion {
    pub(super) fn new_ipc(publisher: Publisher<ipc::Service, PayloadFfi, UserHeaderFfi>) -> Self {
        Self {
            ipc: ManuallyDrop::new(publisher),
        }
    }
    pub(super) fn new_local(
        publisher: Publisher<local::Service, PayloadFfi, UserHeaderFfi>,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(publisher),
        }
    }
}

#[repr(C)]
#[repr(align(16))] // alignment of Option<PublisherUnion>
pub struct iox2_publisher_storage_t {
    internal: [u8; 40], // magic number obtained with size_of::<Option<PublisherUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(PublisherUnion)]
pub struct iox2_publisher_t {
    service_type: iox2_service_type_e,
    value: iox2_publisher_storage_t,
    deleter: fn(*mut iox2_publisher_t),
}

impl iox2_publisher_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: PublisherUnion,
        deleter: fn(*mut iox2_publisher_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_publisher_h_t;
/// The owning handle for `iox2_publisher_t`. Passing the handle to an function transfers the ownership.
pub type iox2_publisher_h = *mut iox2_publisher_h_t;
/// The non-owning handle for `iox2_publisher_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_publisher_h_ref = *const iox2_publisher_h;

impl AssertNonNullHandle for iox2_publisher_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_publisher_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_publisher_h {
    type Target = *mut iox2_publisher_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_publisher_h_ref {
    type Target = *mut iox2_publisher_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition

unsafe fn send_copy<S: Service>(
    publisher: &Publisher<S, PayloadFfi, UserHeaderFfi>,
    data_ptr: *const c_void,
    size_of_element: usize,
    number_of_recipients: *mut usize,
) -> c_int {
    // loan_slice_uninit(1) <= 1 is correct here since it defines the number of
    // slice elements not bytes. The element was set via TypeDetails and has a
    // defined size and alignment.
    let mut sample = match publisher.loan_custom_payload(1) {
        Ok(sample) => sample,
        Err(e) => return e.into_c_int(),
    };

    if sample.payload().len() < size_of_element {
        return iox2_publisher_send_error_e::LOAN_ERROR_EXCEEDS_MAX_LOAN_SIZE as c_int;
    }

    let sample_ptr = sample.payload_mut().as_mut_ptr();
    core::ptr::copy_nonoverlapping(data_ptr, sample_ptr.cast(), size_of_element);
    match sample.assume_init().send() {
        Ok(v) => {
            if !number_of_recipients.is_null() {
                *number_of_recipients = v;
            }
        }
        Err(e) => return e.into_c_int(),
    }

    IOX2_OK
}

unsafe fn send_slice_copy<S: Service>(
    publisher: &Publisher<S, PayloadFfi, UserHeaderFfi>,
    data_ptr: *const c_void,
    size_of_element: usize,
    number_of_elements: usize,
    number_of_recipients: *mut usize,
) -> c_int {
    let mut sample = match publisher.loan_custom_payload(number_of_elements) {
        Ok(sample) => sample,
        Err(e) => return e.into_c_int(),
    };

    let data_len = size_of_element * number_of_elements;
    if sample.payload().len() < data_len {
        return iox2_publisher_send_error_e::LOAN_ERROR_EXCEEDS_MAX_LOAN_SIZE as c_int;
    }

    let sample_ptr = sample.payload_mut().as_mut_ptr();
    core::ptr::copy_nonoverlapping(data_ptr, sample_ptr.cast(), data_len);
    match sample.assume_init().send() {
        Ok(v) => {
            if !number_of_recipients.is_null() {
                *number_of_recipients = v;
            }
        }
        Err(e) => return e.into_c_int(),
    }

    IOX2_OK
}

// BEGIN C API

/// Returns the strategy the publisher follows when a sample cannot be delivered
/// since the subscribers buffer is full.
///
/// # Arguments
///
/// * `handle` obtained by [`iox2_port_factory_publisher_builder_create`](crate::iox2_port_factory_publisher_builder_create)
///
/// Returns [`iox2_unable_to_deliver_strategy_e`].
///
/// # Safety
///
/// * `publisher_handle` is valid and non-null
#[no_mangle]
pub unsafe extern "C" fn iox2_publisher_unable_to_deliver_strategy(
    publisher_handle: iox2_publisher_h_ref,
) -> iox2_unable_to_deliver_strategy_e {
    publisher_handle.assert_non_null();

    let publisher = &mut *publisher_handle.as_type();

    match publisher.service_type {
        iox2_service_type_e::IPC => publisher
            .value
            .as_mut()
            .ipc
            .unable_to_deliver_strategy()
            .into(),
        iox2_service_type_e::LOCAL => publisher
            .value
            .as_mut()
            .local
            .unable_to_deliver_strategy()
            .into(),
    }
}

/// Returns the maximum `[u8]` length that can be loaned in one sample, i.e. the max number of
/// elements in the `[u8]` payload type used by the C binding.
///
/// # Arguments
///
/// * `publisher_handle` obtained by [`iox2_port_factory_publisher_builder_create`](crate::iox2_port_factory_publisher_builder_create)
///
/// Returns the maximum number of elements as a [`c_int`].
///
/// # Safety
///
/// * `publisher_handle` is valid and non-null
#[no_mangle]
pub unsafe extern "C" fn iox2_publisher_max_slice_len(
    publisher_handle: iox2_publisher_h_ref,
) -> c_int {
    publisher_handle.assert_non_null();

    let publisher = &mut *publisher_handle.as_type();
    match publisher.service_type {
        iox2_service_type_e::IPC => publisher.value.as_mut().ipc.max_slice_len() as c_int,
        iox2_service_type_e::LOCAL => publisher.value.as_mut().local.max_slice_len() as c_int,
    }
}

/// Returns the unique port id of the publisher.
///
/// # Arguments
///
/// * `handle` obtained by [`iox2_port_factory_publisher_builder_create`](crate::iox2_port_factory_publisher_builder_create)
/// * `id_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_unique_publisher_id_t`].
///                         If it is a NULL pointer, the storage will be allocated on the heap.
/// * `id_handle_ptr` valid pointer to a [`iox2_unique_publisher_id_h`].
///
/// # Safety
///
/// * `publisher_handle` is valid and non-null
/// * `id` is valid and non-null
#[no_mangle]
pub unsafe extern "C" fn iox2_publisher_id(
    publisher_handle: iox2_publisher_h_ref,
    id_struct_ptr: *mut iox2_unique_publisher_id_t,
    id_handle_ptr: *mut iox2_unique_publisher_id_h,
) {
    publisher_handle.assert_non_null();
    debug_assert!(!id_handle_ptr.is_null());

    fn no_op(_: *mut iox2_unique_publisher_id_t) {}
    let mut deleter: fn(*mut iox2_unique_publisher_id_t) = no_op;
    let mut storage_ptr = id_struct_ptr;
    if id_struct_ptr.is_null() {
        deleter = iox2_unique_publisher_id_t::dealloc;
        storage_ptr = iox2_unique_publisher_id_t::alloc();
    }
    debug_assert!(!storage_ptr.is_null());

    let publisher = &mut *publisher_handle.as_type();

    let id = match publisher.service_type {
        iox2_service_type_e::IPC => publisher.value.as_mut().ipc.id(),
        iox2_service_type_e::LOCAL => publisher.value.as_mut().local.id(),
    };

    (*storage_ptr).init(id, deleter);
    *id_handle_ptr = (*storage_ptr).as_handle();
}

/// Sends a copy of the provided slice data via the publisher.
///
/// # Arguments
///
/// * `publisher_handle` - Handle to the publisher obtained from `iox2_port_factory_publisher_builder_create`
/// * `data_ptr` - Pointer to the start of the slice data to be sent
/// * `size_of_element` - Size of each element in the slice in bytes
/// * `number_of_elements` - Number of elements in the slice
/// * `number_of_recipients` - Optional pointer to store the number of subscribers that received the data
///
/// # Returns
///
/// Returns `IOX2_OK` on success, otherwise an error code from `iox2_publisher_send_error_e`
///
/// # Safety
///
/// * `publisher_handle` must be valid and non-null
/// * `data_ptr` must be a valid pointer to the start of the slice data
/// * `size_of_element` must be the correct size of each element in bytes
/// * `number_of_elements` must accurately represent the number of elements in the slice
/// * `number_of_recipients` can be null, otherwise it must be a valid pointer to a `usize`
#[no_mangle]
pub unsafe extern "C" fn iox2_publisher_send_slice_copy(
    publisher_handle: iox2_publisher_h_ref,
    data_ptr: *const c_void,
    size_of_element: usize,
    number_of_elements: usize,
    number_of_recipients: *mut usize,
) -> c_int {
    publisher_handle.assert_non_null();
    debug_assert!(!data_ptr.is_null());
    debug_assert!(size_of_element != 0);

    let publisher = &mut *publisher_handle.as_type();

    match publisher.service_type {
        iox2_service_type_e::IPC => send_slice_copy(
            &publisher.value.as_mut().ipc,
            data_ptr,
            size_of_element,
            number_of_elements,
            number_of_recipients,
        ),
        iox2_service_type_e::LOCAL => send_slice_copy(
            &publisher.value.as_mut().local,
            data_ptr,
            size_of_element,
            number_of_elements,
            number_of_recipients,
        ),
    }
}

/// Sends a copy of the provided data via the publisher. The data must be copyable via `memcpy`.
///
/// # Arguments
///
/// * `handle` obtained by [`iox2_port_factory_publisher_builder_create`](crate::iox2_port_factory_publisher_builder_create)
/// * `data_ptr` pointer to the payload that shall be transmitted
/// * `data_len` the size of the payload in bytes
/// * `number_of_recipients` (optional) used to store the number of subscriber that received the data
///
/// Return [`IOX2_OK`] on success, otherwise [`iox2_publisher_send_error_e`].
///
/// # Safety
///
/// * `publisher_handle` is valid and non-null
/// * `data_ptr` non-null pointer to a valid position in memory
/// * `data_len` the size of the payload memory
/// * `number_of_recipients` can be null, otherwise a valid pointer to an [`usize`]
#[no_mangle]
pub unsafe extern "C" fn iox2_publisher_send_copy(
    publisher_handle: iox2_publisher_h_ref,
    data_ptr: *const c_void,
    data_len: usize,
    number_of_recipients: *mut usize,
) -> c_int {
    publisher_handle.assert_non_null();
    debug_assert!(!data_ptr.is_null());
    debug_assert!(data_len != 0);

    let publisher = &mut *publisher_handle.as_type();

    match publisher.service_type {
        iox2_service_type_e::IPC => send_copy(
            &publisher.value.as_mut().ipc,
            data_ptr,
            data_len,
            number_of_recipients,
        ),
        iox2_service_type_e::LOCAL => send_copy(
            &publisher.value.as_mut().local,
            data_ptr,
            data_len,
            number_of_recipients,
        ),
    }
}

/// Loans memory from the publishers data segment.
///
/// # Arguments
///
/// * `handle` obtained by [`iox2_port_factory_publisher_builder_create`](crate::iox2_port_factory_publisher_builder_create)
/// * `sample_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_sample_mut_t`].
///    If it is a NULL pointer, the storage will be allocated on the heap.
/// * `sample_handle_ptr` - An uninitialized or dangling [`iox2_sample_mut_h`] handle which will be initialized by this function call if a sample is obtained, otherwise it will be set to NULL.
/// * `number_of_bytes` - The number of bytes to loan from the publisher's payload segment
///
/// Return [`IOX2_OK`] on success, otherwise [`iox2_publisher_loan_error_e`].
///
/// # Safety
///
/// * `publisher_handle` is valid and non-null
/// * The `sample_handle_ptr` is pointing to a valid [`iox2_sample_mut_h`].
#[no_mangle]
pub unsafe extern "C" fn iox2_publisher_loan_slice_uninit(
    publisher_handle: iox2_publisher_h_ref,
    sample_struct_ptr: *mut iox2_sample_mut_t,
    sample_handle_ptr: *mut iox2_sample_mut_h,
    number_of_elements: usize,
) -> c_int {
    publisher_handle.assert_non_null();
    debug_assert!(!sample_handle_ptr.is_null());

    *sample_handle_ptr = std::ptr::null_mut();

    let init_sample_struct_ptr = |sample_struct_ptr: *mut iox2_sample_mut_t| {
        let mut sample_struct_ptr = sample_struct_ptr;
        fn no_op(_: *mut iox2_sample_mut_t) {}
        let mut deleter: fn(*mut iox2_sample_mut_t) = no_op;
        if sample_struct_ptr.is_null() {
            sample_struct_ptr = iox2_sample_mut_t::alloc();
            deleter = iox2_sample_mut_t::dealloc;
        }
        debug_assert!(!sample_struct_ptr.is_null());

        (sample_struct_ptr, deleter)
    };

    let publisher = &mut *publisher_handle.as_type();

    match publisher.service_type {
        iox2_service_type_e::IPC => match publisher
            .value
            .as_ref()
            .ipc
            .loan_custom_payload(number_of_elements)
        {
            Ok(sample) => {
                let (sample_struct_ptr, deleter) = init_sample_struct_ptr(sample_struct_ptr);
                (*sample_struct_ptr).init(
                    publisher.service_type,
                    SampleMutUninitUnion::new_ipc(sample),
                    deleter,
                );
                *sample_handle_ptr = (*sample_struct_ptr).as_handle();
                IOX2_OK
            }
            Err(error) => error.into_c_int(),
        },
        iox2_service_type_e::LOCAL => match publisher
            .value
            .as_ref()
            .local
            .loan_custom_payload(number_of_elements)
        {
            Ok(sample) => {
                let (sample_struct_ptr, deleter) = init_sample_struct_ptr(sample_struct_ptr);
                (*sample_struct_ptr).init(
                    publisher.service_type,
                    SampleMutUninitUnion::new_local(sample),
                    deleter,
                );
                *sample_handle_ptr = (*sample_struct_ptr).as_handle();
                IOX2_OK
            }
            Err(error) => error.into_c_int(),
        },
    }
}

/// Updates all connections to new and obsolete subscriber ports and automatically delivery the history if
/// requested.
///
/// # Arguments
///
/// * `publisher_handle` - Must be a valid [`iox2_publisher_h`]
///   obtained by [`iox2_port_factory_publisher_builder_create`](crate::iox2_port_factory_publisher_builder_create).
///
/// # Safety
///
/// * The `publisher_handle` is still valid after the return of this function and can be use in another function call.
#[no_mangle]
pub unsafe extern "C" fn iox2_publisher_update_connections(
    publisher_handle: iox2_publisher_h_ref,
) -> c_int {
    publisher_handle.assert_non_null();

    let publisher = &mut *publisher_handle.as_type();

    match publisher.service_type {
        iox2_service_type_e::IPC => match publisher.value.as_ref().ipc.update_connections() {
            Ok(()) => IOX2_OK,
            Err(error) => error.into_c_int(),
        },
        iox2_service_type_e::LOCAL => match publisher.value.as_ref().local.update_connections() {
            Ok(()) => IOX2_OK,
            Err(error) => error.into_c_int(),
        },
    }
}

/// This function needs to be called to destroy the publisher!
///
/// # Arguments
///
/// * `publisher_handle` - A valid [`iox2_publisher_h`]
///
/// # Safety
///
/// * The `publisher_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_publisher_t`] can be re-used with a call to
///   [`iox2_port_factory_publisher_builder_create`](crate::iox2_port_factory_publisher_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_publisher_drop(publisher_handle: iox2_publisher_h) {
    publisher_handle.assert_non_null();

    let publisher = &mut *publisher_handle.as_type();

    match publisher.service_type {
        iox2_service_type_e::IPC => {
            ManuallyDrop::drop(&mut publisher.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            ManuallyDrop::drop(&mut publisher.value.as_mut().local);
        }
    }
    (publisher.deleter)(publisher);
}

// END C API
