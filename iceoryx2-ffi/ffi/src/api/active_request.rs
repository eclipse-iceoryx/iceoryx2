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

use super::{iox2_service_type_e, AssertNonNullHandle, HandleToType, PayloadFfi, UserHeaderFfi};
use iceoryx2::active_request::ActiveRequest;
use iceoryx2::prelude::*;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

// BEGIN types definition
pub(super) union ActiveRequestUnion {
    ipc: ManuallyDrop<
        ActiveRequest<ipc::Service, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
    >,
    local: ManuallyDrop<
        ActiveRequest<local::Service, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
    >,
}

impl ActiveRequestUnion {
    pub(super) fn new_ipc(
        active_request: ActiveRequest<
            ipc::Service,
            PayloadFfi,
            UserHeaderFfi,
            PayloadFfi,
            UserHeaderFfi,
        >,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(active_request),
        }
    }
    pub(super) fn new_local(
        active_request: ActiveRequest<
            local::Service,
            PayloadFfi,
            UserHeaderFfi,
            PayloadFfi,
            UserHeaderFfi,
        >,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(active_request),
        }
    }
}

#[repr(C)]
#[repr(align(16))] // alignment of Option<ActiveRequestUnion>
pub struct iox2_active_request_storage_t {
    internal: [u8; 248], // magic number obtained with size_of::<Option<ActiveRequestUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(ActiveRequestUnion)]
pub struct iox2_active_request_t {
    service_type: iox2_service_type_e,
    value: iox2_active_request_storage_t,
    deleter: fn(*mut iox2_active_request_t),
}

impl iox2_active_request_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: ActiveRequestUnion,
        deleter: fn(*mut iox2_active_request_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_active_request_h_t;
/// The owning handle for `iox2_active_request_t`. Passing the handle to an function transfers the ownership.
pub type iox2_active_request_h = *mut iox2_active_request_h_t;
/// The non-owning handle for `iox2_active_request_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_active_request_h_ref = *const iox2_active_request_h;

impl AssertNonNullHandle for iox2_active_request_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_active_request_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_active_request_h {
    type Target = *mut iox2_active_request_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_active_request_h_ref {
    type Target = *mut iox2_active_request_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END types definition
