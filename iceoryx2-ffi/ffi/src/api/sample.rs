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

use crate::api::{iox2_service_type_e, HandleToType, PayloadFfi, UserHeaderFfi};
use crate::{c_size_t, iox2_publish_subscribe_header_h, iox2_publish_subscribe_header_t};

use iceoryx2::prelude::*;
use iceoryx2::sample::Sample;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::ffi::c_void;
use core::mem::ManuallyDrop;

// BEGIN types definition

pub(super) union SampleUnion {
    ipc: ManuallyDrop<Sample<ipc::Service, PayloadFfi, UserHeaderFfi>>,
    local: ManuallyDrop<Sample<local::Service, PayloadFfi, UserHeaderFfi>>,
}

impl SampleUnion {
    pub(super) fn new_ipc(sample: Sample<ipc::Service, PayloadFfi, UserHeaderFfi>) -> Self {
        Self {
            ipc: ManuallyDrop::new(sample),
        }
    }
    pub(super) fn new_local(sample: Sample<local::Service, PayloadFfi, UserHeaderFfi>) -> Self {
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

pub struct iox2_sample_h_ref_t;
/// The non-owning handle for `iox2_sample_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_sample_h_ref = *mut iox2_sample_h_ref_t;

impl HandleToType for iox2_sample_h {
    type Target = *mut iox2_sample_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_sample_h_ref {
    type Target = *mut iox2_sample_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

// END type definition

// BEGIN C API

/// This function casts an owning [`iox2_sample_h`] into a non-owning [`iox2_sample_h_ref`]
///
/// # Arguments
///
/// * `handle` obtained by [`iox2_subscriber_receive()`](crate::iox2_subscriber_receive())
///
/// Returns a [`iox2_sample_h_ref`]
///
/// # Safety
///
/// * The `handle` must be a valid handle.
/// * The `handle` is still valid after the call to this function.
#[no_mangle]
pub unsafe extern "C" fn iox2_cast_sample_h_ref(handle: iox2_sample_h) -> iox2_sample_h_ref {
    debug_assert!(!handle.is_null());
    (*handle.as_type()).as_h_refandle() as *mut _ as _
}

/// cbindgen:ignore
/// Internal API - do not use
/// # Safety
///
/// * `source_struct_ptr` must not be `null` and the struct it is pointing to must be initialized and valid, i.e. not moved or dropped.
/// * `dest_struct_ptr` must not be `null` and the struct it is pointing to must not contain valid data, i.e. initialized. It can be moved or dropped, though.
/// * `dest_handle_ptr` must not be `null`
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn iox2_sample_move(
    source_struct_ptr: *mut iox2_sample_t,
    dest_struct_ptr: *mut iox2_sample_t,
    dest_handle_ptr: *mut iox2_sample_h,
) {
    debug_assert!(!source_struct_ptr.is_null());
    debug_assert!(!dest_struct_ptr.is_null());
    debug_assert!(!dest_handle_ptr.is_null());

    let source = &mut *source_struct_ptr;
    let dest = &mut *dest_struct_ptr;

    dest.service_type = source.service_type;
    dest.value.init(
        source
            .value
            .as_option_mut()
            .take()
            .expect("Source must have a valid sample"),
    );
    dest.deleter = source.deleter;

    *dest_handle_ptr = (*dest_struct_ptr).as_handle();
}

/// Acquires the samples header.
///
/// # Safety
///
/// * `handle` obtained by [`iox2_subscriber_receive()`](crate::iox2_subscriber_receive())
/// * `header_struct_ptr` - Must be either a NULL pointer or a pointer to a valid
///     [`iox2_publish_subscribe_header_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `header_handle_ptr` valid pointer to a [`iox2_publish_subscribe_header_h`].
#[no_mangle]
pub unsafe extern "C" fn iox2_sample_header(
    handle: iox2_sample_h_ref,
    header_struct_ptr: *mut iox2_publish_subscribe_header_t,
    header_handle_ptr: *mut iox2_publish_subscribe_header_h,
) {
    debug_assert!(!handle.is_null());
    debug_assert!(!header_handle_ptr.is_null());

    fn no_op(_: *mut iox2_publish_subscribe_header_t) {}
    let mut deleter: fn(*mut iox2_publish_subscribe_header_t) = no_op;
    let mut storage_ptr = header_struct_ptr;
    if header_struct_ptr.is_null() {
        deleter = iox2_publish_subscribe_header_t::dealloc;
        storage_ptr = iox2_publish_subscribe_header_t::alloc();
    }
    debug_assert!(!storage_ptr.is_null());

    let sample = &mut *handle.as_type();

    let header = *match sample.service_type {
        iox2_service_type_e::IPC => sample.value.as_mut().ipc.header(),
        iox2_service_type_e::LOCAL => sample.value.as_mut().local.header(),
    };

    (*storage_ptr).init(header, deleter);
    *header_handle_ptr = (*storage_ptr).as_handle();
}

/// Acquires the samples user header.
///
/// # Safety
///
/// * `handle` obtained by [`iox2_subscriber_receive()`](crate::iox2_subscriber_receive())
/// * `header_ptr` a valid, non-null pointer pointing to a [`*const c_void`] pointer.
#[no_mangle]
pub unsafe extern "C" fn iox2_sample_user_header(
    sample_handle: iox2_sample_h_ref,
    header_ptr: *mut *const c_void,
) {
    debug_assert!(!sample_handle.is_null());
    debug_assert!(!header_ptr.is_null());

    let sample = &mut *sample_handle.as_type();

    let header = match sample.service_type {
        iox2_service_type_e::IPC => sample.value.as_mut().ipc.user_header(),
        iox2_service_type_e::LOCAL => sample.value.as_mut().local.user_header(),
    };

    *header_ptr = (header as *const UserHeaderFfi).cast();
}

/// Acquires the samples payload.
///
/// # Safety
///
/// * `handle` obtained by [`iox2_subscriber_receive()`](crate::iox2_subscriber_receive())
/// * `payload_ptr` a valid, non-null pointer pointing to a [`*const c_void`] pointer.
/// * `payload_len` (optional) either a null poitner or a valid pointer pointing to a [`c_size_t`].
#[no_mangle]
pub unsafe extern "C" fn iox2_sample_payload(
    sample_handle: iox2_sample_h_ref,
    payload_ptr: *mut *const c_void,
    payload_len: *mut c_size_t,
) {
    debug_assert!(!sample_handle.is_null());
    debug_assert!(!payload_ptr.is_null());

    let sample = &mut *sample_handle.as_type();

    let payload = match sample.service_type {
        iox2_service_type_e::IPC => sample.value.as_mut().ipc.payload(),
        iox2_service_type_e::LOCAL => sample.value.as_mut().local.payload(),
    };

    *payload_ptr = payload.as_ptr().cast();
    if !payload_len.is_null() {
        *payload_len = payload.len();
    }
}

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
