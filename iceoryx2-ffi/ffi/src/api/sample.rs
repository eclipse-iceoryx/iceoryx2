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

use iceoryx2::prelude::*;
use iceoryx2::sample::Sample;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::mem::ManuallyDrop;

// BEGIN types definition

pub(super) union SampleUnion {
    ipc: ManuallyDrop<Sample<zero_copy::Service, PayloadFfi, NoUserHeaderFfi>>,
    local: ManuallyDrop<Sample<process_local::Service, PayloadFfi, NoUserHeaderFfi>>,
}

impl SampleUnion {
    pub(super) fn new_ipc(sample: Sample<zero_copy::Service, PayloadFfi, NoUserHeaderFfi>) -> Self {
        Self {
            ipc: ManuallyDrop::new(sample),
        }
    }
    pub(super) fn new_local(
        sample: Sample<process_local::Service, PayloadFfi, NoUserHeaderFfi>,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(sample),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<SampleUnion>
pub struct iox2_sample_storage_t {
    internal: [u8; 80], // magic number obtained with size_of::<Option<SampleUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(SampleUnion)]
pub struct iox2_sample_t {
    service_type: iox2_service_type_e,
    value: iox2_sample_storage_t,
    deleter: fn(*mut iox2_sample_t),
}

impl iox2_sample_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: SampleUnion,
        deleter: fn(*mut iox2_sample_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_sample_h_t;
/// The owning handle for `iox2_sample_t`. Passing the handle to an function transfers the ownership.
pub type iox2_sample_h = *mut iox2_sample_h_t;

pub struct iox2_sample_ref_h_t;
/// The non-owning handle for `iox2_sample_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_sample_ref_h = *mut iox2_sample_ref_h_t;

impl HandleToType for iox2_sample_h {
    type Target = *mut iox2_sample_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_sample_ref_h {
    type Target = *mut iox2_sample_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

// END type definition

// BEGIN C API

/// This function needs to be called to destroy the sample!
///
/// # Arguments
///
/// * `sample_handle` - A valid [`iox2_sample_h`]
///
/// # Safety
///
/// * The `sample_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_sample_t`] can be re-used with a call to
///   [`iox2_subscriber_receive`](crate::iox2_subscriber_receive)!
#[no_mangle]
pub unsafe extern "C" fn iox2_sample_drop(sample_handle: iox2_sample_h) {
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
