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

use crate::api::{iox2_service_type_e, HandleToType, NoUserHeaderFfi, PayloadFfi};
use crate::IOX2_OK;

use iceoryx2::port::publisher::{Publisher, PublisherLoanError, PublisherSendError};
use iceoryx2::prelude::*;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

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

#[no_mangle]
pub unsafe extern "C" fn iox2_cast_publisher_ref_h(
    handle: iox2_publisher_h,
) -> iox2_publisher_ref_h {
    debug_assert!(!handle.is_null());

    (*handle.as_type()).as_ref_handle() as *mut _ as _
}

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

use core::time::Duration;
use iceoryx2_bb_log::set_log_level;

use super::IntoCInt;

const CYCLE_TIME: Duration = Duration::from_secs(1);

#[no_mangle]
pub extern "C" fn run_publisher(seconds: u32) -> i32 {
    set_log_level(iceoryx2_bb_log::LogLevel::Info);

    let service_name = ServiceName::new("Hello/from/C");
    let node = NodeBuilder::new().create::<zero_copy::Service>();

    if service_name.is_err() || node.is_err() {
        return -1;
    }

    let service_name = service_name.unwrap();
    let node = node.unwrap();

    let service = node
        .service_builder(&service_name)
        .publish_subscribe::<u64>()
        .open_or_create();

    if service.is_err() {
        return -1;
    }

    let service = service.unwrap();

    let publisher = service.publisher_builder().create();

    if publisher.is_err() {
        return -1;
    }

    let publisher = publisher.unwrap();

    let mut counter: u64 = 0;

    let mut remaining_seconds = seconds;

    while let NodeEvent::Tick = node.wait(CYCLE_TIME) {
        counter += 1;
        let sample = publisher.loan_uninit();

        if sample.is_err() {
            return -1;
        }

        let sample = sample.unwrap();

        let sample = sample.write_payload(counter);

        if sample.send().is_err() {
            return -1;
        }

        println!("Send sample {} ...", counter);

        remaining_seconds = remaining_seconds.saturating_sub(1);
        if remaining_seconds == 0 {
            break;
        }
    }

    println!("exit");

    0
}
