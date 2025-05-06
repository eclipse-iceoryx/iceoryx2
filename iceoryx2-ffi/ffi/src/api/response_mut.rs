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

use core::mem::ManuallyDrop;

use iceoryx2::{prelude::*, response_mut_uninit::ResponseMutUninit};
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use super::{
    iox2_service_type_e, AssertNonNullHandle, HandleToType, UninitPayloadFfi, UserHeaderFfi,
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
