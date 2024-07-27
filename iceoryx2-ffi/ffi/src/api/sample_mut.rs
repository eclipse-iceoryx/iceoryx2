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

use crate::api::{iox2_service_type_e, HandleToType, IntoCInt, NoUserHeaderFfi};
use crate::{c_size_t, IOX2_OK};

use iceoryx2::prelude::*;
use iceoryx2::sample_mut::SampleMut;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::ffi::{c_int, c_void};
use core::mem::ManuallyDrop;

use super::UninitPayloadFfi;

// BEGIN types definition

pub(super) union SampleMutUnion {
    ipc: ManuallyDrop<SampleMut<zero_copy::Service, UninitPayloadFfi, NoUserHeaderFfi>>,
    local: ManuallyDrop<SampleMut<process_local::Service, UninitPayloadFfi, NoUserHeaderFfi>>,
}

impl SampleMutUnion {
    pub(super) fn new_ipc(
        sample: SampleMut<zero_copy::Service, UninitPayloadFfi, NoUserHeaderFfi>,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(sample),
        }
    }
    pub(super) fn new_local(
        sample: SampleMut<process_local::Service, UninitPayloadFfi, NoUserHeaderFfi>,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(sample),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<SampleMutUnion>
pub struct iox2_sample_mut_storage_t {
    internal: [u8; 56], // magic number obtained with size_of::<Option<SampleMutUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(SampleMutUnion)]
pub struct iox2_sample_mut_t {
    service_type: iox2_service_type_e,
    value: iox2_sample_mut_storage_t,
    deleter: fn(*mut iox2_sample_mut_t),
}

impl iox2_sample_mut_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: SampleMutUnion,
        deleter: fn(*mut iox2_sample_mut_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_sample_mut_h_t;
/// The owning handle for `iox2_sample_mut_t`. Passing the handle to an function transfers the ownership.
pub type iox2_sample_mut_h = *mut iox2_sample_mut_h_t;

pub struct iox2_sample_mut_ref_h_t;
/// The non-owning handle for `iox2_sample_mut_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_sample_mut_ref_h = *mut iox2_sample_mut_ref_h_t;

impl HandleToType for iox2_sample_mut_h {
    type Target = *mut iox2_sample_mut_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_sample_mut_ref_h {
    type Target = *mut iox2_sample_mut_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

// END type definition

// BEGIN C API

/// This function casts an owning [`iox2_sample_mut_h`] into a non-owning [`iox2_sample_mut_ref_h`]
///
/// # Arguments
///
/// * `handle` obtained by [`iox2_publisher_loan()`](crate::iox2_publisher_loan())
///
/// Returns a [`iox2_sample_mut_ref_h`]
///
/// # Safety
///
/// * The `handle` must be a valid handle.
/// * The `handle` is still valid after the call to this function.
#[no_mangle]
pub unsafe extern "C" fn iox2_cast_sample_mut_ref_h(
    handle: iox2_sample_mut_h,
) -> iox2_sample_mut_ref_h {
    debug_assert!(!handle.is_null());
    (*handle.as_type()).as_ref_handle() as *mut _ as _
}

/// Acquires the samples mutable payload.
///
/// # Safety
///
/// * `handle` obtained by [`iox2_publisher_loan()`](crate::iox2_publisher_loan())
/// * `payload_ptr` a valid, non-null pointer pointing to a [`*const c_void`] pointer.
/// * `payload_len` a valid, non-null pointer pointing to a [`c_size_t`].
#[no_mangle]
pub unsafe extern "C" fn iox2_sample_mut_payload_mut(
    sample_handle: iox2_sample_mut_ref_h,
    payload_ptr: *mut *mut c_void,
    payload_len: *mut c_size_t,
) {
    debug_assert!(!sample_handle.is_null());
    debug_assert!(!payload_ptr.is_null());
    debug_assert!(!payload_len.is_null());

    let sample = &mut *sample_handle.as_type();

    let payload = match sample.service_type {
        iox2_service_type_e::IPC => sample.value.as_mut().ipc.payload_mut(),
        iox2_service_type_e::LOCAL => sample.value.as_mut().local.payload_mut(),
    };

    *payload_ptr = payload.as_mut_ptr().cast();
    *payload_len = payload.len();
}

/// Acquires the samples payload.
///
/// # Safety
///
/// * `handle` obtained by [`iox2_publisher_loan()`](crate::iox2_publisher_loan())
/// * `payload_ptr` a valid, non-null pointer pointing to a [`*const c_void`] pointer.
/// * `payload_len` a valid, non-null pointer pointing to a [`c_size_t`].
#[no_mangle]
pub unsafe extern "C" fn iox2_sample_mut_payload(
    sample_handle: iox2_sample_mut_ref_h,
    payload_ptr: *mut *const c_void,
    payload_len: *mut c_size_t,
) {
    debug_assert!(!sample_handle.is_null());
    debug_assert!(!payload_ptr.is_null());
    debug_assert!(!payload_len.is_null());

    let sample = &mut *sample_handle.as_type();

    let payload = match sample.service_type {
        iox2_service_type_e::IPC => sample.value.as_mut().ipc.payload(),
        iox2_service_type_e::LOCAL => sample.value.as_mut().local.payload(),
    };

    *payload_ptr = payload.as_ptr().cast();
    *payload_len = payload.len();
}

/// Takes the ownership of the sample and sends it
///
/// # Safety
///
/// * `handle` obtained by [`iox2_publisher_loan()`](crate::iox2_publisher_loan())
/// * `number_of_recipients`, can be null or must point to a valid [`c_size_t`] to store the number
///                 of subscribers that received the sample
#[no_mangle]
pub unsafe extern "C" fn iox2_sample_mut_send(
    sample_handle: iox2_sample_mut_h,
    number_of_recipients: *mut c_size_t,
) -> c_int {
    debug_assert!(!sample_handle.is_null());

    let sample_struct = &mut *sample_handle.as_type();
    let service_type = sample_struct.service_type;

    let sample = sample_struct
        .value
        .as_option_mut()
        .take()
        .unwrap_or_else(|| panic!("Trying to send an already sent sample!"));
    (sample_struct.deleter)(sample_struct);

    match service_type {
        iox2_service_type_e::IPC => {
            let sample = ManuallyDrop::into_inner(sample.ipc);
            match sample.assume_init().send() {
                Ok(v) => {
                    if !number_of_recipients.is_null() {
                        *number_of_recipients = v;
                    }
                }
                Err(e) => {
                    (sample_struct.deleter)(sample_struct);
                    return e.into_c_int();
                }
            }
        }
        iox2_service_type_e::LOCAL => {
            let sample = ManuallyDrop::into_inner(sample.local);
            match sample.assume_init().send() {
                Ok(v) => {
                    if !number_of_recipients.is_null() {
                        *number_of_recipients = v;
                    }
                }
                Err(e) => {
                    (sample_struct.deleter)(sample_struct);
                    return e.into_c_int();
                }
            }
        }
    }

    IOX2_OK
}

/// This function needs to be called to destroy the sample!
///
/// # Arguments
///
/// * `sample_handle` - A valid [`iox2_sample_mut_h`]
///
/// # Safety
///
/// * The `sample_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_sample_mut_t`] can be re-used with a call to
///   [`iox2_subscriber_receive`](crate::iox2_subscriber_receive)!
#[no_mangle]
pub unsafe extern "C" fn iox2_sample_mut_drop(sample_handle: iox2_sample_mut_h) {
    debug_assert!(!sample_handle.is_null());

    let sample = &mut *sample_handle.as_type();

    match sample.service_type {
        iox2_service_type_e::IPC => {
            ManuallyDrop::drop(&mut sample.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            ManuallyDrop::drop(&mut sample.value.as_mut().local);
        }
    }
    (sample.deleter)(sample);
}

// END C API
