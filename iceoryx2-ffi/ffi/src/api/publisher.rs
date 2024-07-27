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

use crate::api::{iox2_service_type_e, HandleToType, NoUserHeaderFfi, PayloadFfi, SampleMutUnion};
use crate::IOX2_OK;

use iceoryx2::port::publisher::{Publisher, PublisherLoanError, PublisherSendError};
use iceoryx2::prelude::*;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use super::{c_size_t, iox2_sample_mut_h, iox2_sample_mut_t, IntoCInt};

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
            PublisherLoanError::OutOfMemory => {
                iox2_publisher_send_error_e::LOAN_ERROR_OUT_OF_MEMORY
            }
            PublisherLoanError::ExceedsMaxLoanedSamples => {
                iox2_publisher_send_error_e::LOAN_ERROR_EXCEEDS_MAX_LOANED_SAMPLES
            }
            PublisherLoanError::ExceedsMaxLoanSize => {
                iox2_publisher_send_error_e::LOAN_ERROR_EXCEEDS_MAX_LOAN_SIZE
            }
            PublisherLoanError::InternalFailure => {
                iox2_publisher_send_error_e::LOAN_ERROR_INTERNAL_FAILURE
            }
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
    ipc: ManuallyDrop<Publisher<zero_copy::Service, PayloadFfi, NoUserHeaderFfi>>,
    local: ManuallyDrop<Publisher<process_local::Service, PayloadFfi, NoUserHeaderFfi>>,
}

impl PublisherUnion {
    pub(super) fn new_ipc(
        publisher: Publisher<zero_copy::Service, PayloadFfi, NoUserHeaderFfi>,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(publisher),
        }
    }
    pub(super) fn new_local(
        publisher: Publisher<process_local::Service, PayloadFfi, NoUserHeaderFfi>,
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

pub struct iox2_publisher_ref_h_t;
/// The non-owning handle for `iox2_publisher_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_publisher_ref_h = *mut iox2_publisher_ref_h_t;

impl HandleToType for iox2_publisher_h {
    type Target = *mut iox2_publisher_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_publisher_ref_h {
    type Target = *mut iox2_publisher_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

// END type definition

unsafe fn send_copy<S: Service>(
    publisher: &Publisher<S, PayloadFfi, NoUserHeaderFfi>,
    data_ptr: *const c_void,
    data_len: usize,
    number_of_recipients: *mut usize,
) -> c_int {
    // loan_slice_uninit(1) <= 1 is correct here since it defines the number of
    // slice elements not bytes. The element was set via TypeDetails and has a
    // defined size and alignment.
    let mut sample = match publisher.loan_slice_uninit(1) {
        Ok(sample) => sample,
        Err(e) => return e.into_c_int(),
    };

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

/// This function casts an owning [`iox2_publisher_h`] into a non-owning [`iox2_publisher_ref_h`]
///
/// # Arguments
///
/// * `handle` obtained by [`iox2_port_factory_publisher_builder_create`](crate::iox2_port_factory_publisher_builder_create)
///
/// Returns a [`iox2_publisher_ref_h`]
///
/// # Safety
///
/// * The `handle` must be a valid handle.
/// * The `handle` is still valid after the call to this function.
#[no_mangle]
pub unsafe extern "C" fn iox2_cast_publisher_ref_h(
    handle: iox2_publisher_h,
) -> iox2_publisher_ref_h {
    debug_assert!(!handle.is_null());

    (*handle.as_type()).as_ref_handle() as *mut _ as _
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
/// * `publisher_handle` is valid, non-null and was obtained via [`iox2_cast_publisher_ref_h`]
/// * `data_ptr` non-null pointer to a valid position in memory
/// * `data_len` the size of the payload memory
/// * `number_of_recipients` can be null, otherwise a valid pointer to an [`usize`]
#[no_mangle]
pub unsafe extern "C" fn iox2_publisher_send_copy(
    publisher_handle: iox2_publisher_ref_h,
    data_ptr: *const c_void,
    data_len: usize,
    number_of_recipients: *mut usize,
) -> c_int {
    debug_assert!(!publisher_handle.is_null());
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
/// * `number_of_elements` defines the number of elements that shall be loaned. The elements were
///    defined via [`iox2_service_builder_pub_sub_set_payload_type_details()`](crate::iox2_service_builder_pub_sub_set_payload_type_details).
/// * `sample_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_sample_mut_t`].
///    If it is a NULL pointer, the storage will be allocated on the heap.
/// * `sample_handle_ptr` - An uninitialized or dangling [`iox2_sample_mut_h`] handle which will be initialized by this function call if a sample is obtained, otherwise it will be set to NULL.
///
/// Return [`IOX2_OK`] on success, otherwise [`iox2_publisher_loan_error_e`].
///
/// # Safety
///
/// * `publisher_handle` is valid, non-null and was obtained via [`iox2_cast_publisher_ref_h`]
/// * The `sample_handle_ptr` is pointing to a valid [`iox2_sample_mut_h`].
#[no_mangle]
pub unsafe extern "C" fn iox2_publisher_loan(
    publisher_handle: iox2_publisher_ref_h,
    number_of_elements: c_size_t,
    sample_struct_ptr: *mut iox2_sample_mut_t,
    sample_handle_ptr: *mut iox2_sample_mut_h,
) -> c_int {
    debug_assert!(!publisher_handle.is_null());
    debug_assert!(!sample_handle_ptr.is_null());

    *sample_handle_ptr = std::ptr::null_mut();

    let mut sample_struct_ptr = sample_struct_ptr;
    fn no_op(_: *mut iox2_sample_mut_t) {}
    let mut deleter: fn(*mut iox2_sample_mut_t) = no_op;
    if sample_struct_ptr.is_null() {
        sample_struct_ptr = iox2_sample_mut_t::alloc();
        deleter = iox2_sample_mut_t::dealloc;
    }
    debug_assert!(!sample_struct_ptr.is_null());

    let publisher = &mut *publisher_handle.as_type();

    match publisher.service_type {
        iox2_service_type_e::IPC => match publisher
            .value
            .as_ref()
            .ipc
            .loan_slice_uninit(number_of_elements)
        {
            Ok(sample) => {
                (*sample_struct_ptr).init(
                    publisher.service_type,
                    SampleMutUnion::new_ipc(sample),
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
            .loan_slice_uninit(number_of_elements)
        {
            Ok(sample) => {
                (*sample_struct_ptr).init(
                    publisher.service_type,
                    SampleMutUnion::new_local(sample),
                    deleter,
                );
                *sample_handle_ptr = (*sample_struct_ptr).as_handle();
                IOX2_OK
            }
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
    debug_assert!(!publisher_handle.is_null());

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
