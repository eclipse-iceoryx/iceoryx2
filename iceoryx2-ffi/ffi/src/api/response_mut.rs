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

use core::{ffi::c_int, ffi::c_void, mem::ManuallyDrop};

use iceoryx2::{prelude::*, response_mut_uninit::ResponseMutUninit};
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use crate::{api::IntoCInt, IOX2_OK};

use super::{
    c_size_t, iox2_response_header_h, iox2_response_header_t, iox2_service_type_e,
    AssertNonNullHandle, HandleToType, UninitPayloadFfi, UserHeaderFfi,
};

// BEGIN types definition
pub(super) union ResponseMutUninitUnion {
    ipc: ManuallyDrop<ResponseMutUninit<ipc::Service, UninitPayloadFfi, UserHeaderFfi>>,
    local: ManuallyDrop<ResponseMutUninit<local::Service, UninitPayloadFfi, UserHeaderFfi>>,
}

impl ResponseMutUninitUnion {
    pub(super) fn new_ipc(
        sample: ResponseMutUninit<ipc::Service, UninitPayloadFfi, UserHeaderFfi>,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(sample),
        }
    }
    pub(super) fn new_local(
        sample: ResponseMutUninit<local::Service, UninitPayloadFfi, UserHeaderFfi>,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(sample),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<ResponseMutUninitUnion>
pub struct iox2_response_mut_storage_t {
    internal: [u8; 88], // magic number obtained with size_of::<Option<ResponseMutUninitUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(ResponseMutUninitUnion)]
pub struct iox2_response_mut_t {
    service_type: iox2_service_type_e,
    value: iox2_response_mut_storage_t,
    deleter: fn(*mut iox2_response_mut_t),
}

impl iox2_response_mut_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: ResponseMutUninitUnion,
        deleter: fn(*mut iox2_response_mut_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_response_mut_h_t;
/// The owning handle for `iox2_response_mut_t`. Passing the handle to an function transfers the ownership.
pub type iox2_response_mut_h = *mut iox2_response_mut_h_t;
/// The non-owning handle for `iox2_response_mut_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_response_mut_h_ref = *const iox2_response_mut_h;

impl AssertNonNullHandle for iox2_response_mut_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_response_mut_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_response_mut_h {
    type Target = *mut iox2_response_mut_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_response_mut_h_ref {
    type Target = *mut iox2_response_mut_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition

// BEGIN C API
#[no_mangle]
pub unsafe extern "C" fn iox2_response_mut_header(
    handle: iox2_response_mut_h_ref,
    header_struct_ptr: *mut iox2_response_header_t,
    header_handle_ptr: *mut iox2_response_header_h,
) {
    handle.assert_non_null();
    debug_assert!(!header_handle_ptr.is_null());

    fn no_op(_: *mut iox2_response_header_t) {}
    let mut deleter: fn(*mut iox2_response_header_t) = no_op;
    let mut storage_ptr = header_struct_ptr;
    if header_struct_ptr.is_null() {
        deleter = iox2_response_header_t::dealloc;
        storage_ptr = iox2_response_header_t::alloc();
    }
    debug_assert!(!storage_ptr.is_null());

    let response = &mut *handle.as_type();

    let header = *match response.service_type {
        iox2_service_type_e::IPC => response.value.as_mut().ipc.header(),
        iox2_service_type_e::LOCAL => response.value.as_mut().local.header(),
    };

    (*storage_ptr).init(header, deleter);
    *header_handle_ptr = (*storage_ptr).as_handle();
}

#[no_mangle]
pub unsafe extern "C" fn iox2_response_mut_user_header(
    handle: iox2_response_mut_h_ref,
    header_ptr: *mut *const c_void,
) {
    handle.assert_non_null();
    debug_assert!(!header_ptr.is_null());

    let response = &mut *handle.as_type();

    let header = match response.service_type {
        iox2_service_type_e::IPC => response.value.as_mut().ipc.user_header(),
        iox2_service_type_e::LOCAL => response.value.as_mut().local.user_header(),
    };

    *header_ptr = (header as *const UserHeaderFfi).cast();
}

#[no_mangle]
pub unsafe extern "C" fn iox2_response_mut_user_header_mut(
    handle: iox2_response_mut_h_ref,
    header_ptr: *mut *mut c_void,
) {
    handle.assert_non_null();
    debug_assert!(!header_ptr.is_null());

    let response = &mut *handle.as_type();

    let header = match response.service_type {
        iox2_service_type_e::IPC => response.value.as_mut().ipc.user_header_mut(),
        iox2_service_type_e::LOCAL => response.value.as_mut().local.user_header_mut(),
    };

    *header_ptr = (header as *mut UserHeaderFfi).cast();
}

#[no_mangle]
pub unsafe extern "C" fn iox2_response_mut_payload(
    handle: iox2_response_mut_h_ref,
    payload_ptr: *mut *const c_void,
    number_of_elements: *mut c_size_t,
) {
    handle.assert_non_null();
    debug_assert!(!payload_ptr.is_null());

    let response = &mut *handle.as_type();
    let payload = response.value.as_mut().ipc.payload_mut();

    match response.service_type {
        iox2_service_type_e::IPC => {
            *payload_ptr = payload.as_mut_ptr().cast();
        }
        iox2_service_type_e::LOCAL => {
            *payload_ptr = payload.as_mut_ptr().cast();
        }
    };

    if !number_of_elements.is_null() {
        *number_of_elements =
            response.value.as_mut().local.header().number_of_elements() as c_size_t;
    }
}

#[no_mangle]
pub unsafe extern "C" fn iox2_response_mut_payload_mut(
    handle: iox2_response_mut_h_ref,
    payload_ptr: *mut *mut c_void,
    number_of_elements: *mut c_size_t,
) {
    handle.assert_non_null();
    debug_assert!(!payload_ptr.is_null());

    let response = &mut *handle.as_type();
    let payload = response.value.as_mut().ipc.payload_mut();

    match response.service_type {
        iox2_service_type_e::IPC => {
            *payload_ptr = payload.as_mut_ptr().cast();
        }
        iox2_service_type_e::LOCAL => {
            *payload_ptr = payload.as_mut_ptr().cast();
        }
    };

    if !number_of_elements.is_null() {
        *number_of_elements =
            response.value.as_mut().local.header().number_of_elements() as c_size_t;
    }
}

#[no_mangle]
pub unsafe extern "C" fn iox2_response_mut_send(response_handle: iox2_response_mut_h) -> c_int {
    debug_assert!(!response_handle.is_null());

    let response_struct = &mut *response_handle.as_type();
    let service_type = response_struct.service_type;

    let response = response_struct
        .value
        .as_option_mut()
        .take()
        .unwrap_or_else(|| panic!("Trying to send an already sent response!"));
    (response_struct.deleter)(response_struct);

    match service_type {
        iox2_service_type_e::IPC => {
            let response = ManuallyDrop::into_inner(response.ipc);
            match response.assume_init().send() {
                Ok(()) => IOX2_OK,
                Err(e) => e.into_c_int(),
            }
        }
        iox2_service_type_e::LOCAL => {
            let response = ManuallyDrop::into_inner(response.local);
            match response.assume_init().send() {
                Ok(()) => IOX2_OK,
                Err(e) => e.into_c_int(),
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn iox2_response_mut_drop(response_handle: iox2_response_mut_h) {
    debug_assert!(!response_handle.is_null());

    let response = &mut *response_handle.as_type();

    match response.service_type {
        iox2_service_type_e::IPC => {
            ManuallyDrop::drop(&mut response.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            ManuallyDrop::drop(&mut response.value.as_mut().local);
        }
    }
    (response.deleter)(response);
}
// END C API
