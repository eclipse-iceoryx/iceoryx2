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

// BEGIN types definition

use core::mem::ManuallyDrop;
use iceoryx2::prelude::*;
use iceoryx2::response::Response;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use super::{iox2_service_type_e, AssertNonNullHandle, HandleToType, PayloadFfi, UserHeaderFfi};

pub(super) union ResponseUnion {
    ipc: ManuallyDrop<Response<ipc::Service, PayloadFfi, UserHeaderFfi>>,
    local: ManuallyDrop<Response<local::Service, PayloadFfi, UserHeaderFfi>>,
}

impl ResponseUnion {
    pub(super) fn new_ipc(sample: Response<ipc::Service, PayloadFfi, UserHeaderFfi>) -> Self {
        Self {
            ipc: ManuallyDrop::new(sample),
        }
    }
    pub(super) fn new_local(sample: Response<local::Service, PayloadFfi, UserHeaderFfi>) -> Self {
        Self {
            local: ManuallyDrop::new(sample),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<ResponseUnion>
pub struct iox2_response_storage_t {
    internal: [u8; 80], // magic number obtained with size_of::<Option<ResponseUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(ResponseUnion)]
pub struct iox2_response_t {
    service_type: iox2_service_type_e,
    value: iox2_response_storage_t,
    deleter: fn(*mut iox2_response_t),
}

impl iox2_response_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: ResponseUnion,
        deleter: fn(*mut iox2_response_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_response_h_t;
/// The owning handle for `iox2_sample_t`. Passing the handle to an function transfers the ownership.
pub type iox2_response_h = *mut iox2_response_h_t;
/// The non-owning handle for `iox2_response_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_response_h_ref = *const iox2_response_h;

impl AssertNonNullHandle for iox2_response_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_response_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_response_h {
    type Target = *mut iox2_response_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_response_h_ref {
    type Target = *mut iox2_response_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition
