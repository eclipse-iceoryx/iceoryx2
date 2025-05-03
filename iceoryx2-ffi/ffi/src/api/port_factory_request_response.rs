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

// BEGIN types definition

use core::mem::ManuallyDrop;
use iceoryx2::prelude::*;
use iceoryx2::service::port_factory::request_response::PortFactory;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use super::{iox2_service_type_e, AssertNonNullHandle, HandleToType, PayloadFfi, UserHeaderFfi};

// BEGIN types definition
pub(super) union PortFactoryRequestResponseUnion {
    ipc: ManuallyDrop<
        PortFactory<ipc::Service, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
    >,
    local: ManuallyDrop<
        PortFactory<local::Service, PayloadFfi, UserHeaderFfi, PayloadFfi, UserHeaderFfi>,
    >,
}

impl PortFactoryRequestResponseUnion {
    pub(super) fn new_ipc(
        port_factory: PortFactory<
            ipc::Service,
            PayloadFfi,
            UserHeaderFfi,
            PayloadFfi,
            UserHeaderFfi,
        >,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(port_factory),
        }
    }
    pub(super) fn new_local(
        port_factory: PortFactory<
            local::Service,
            PayloadFfi,
            UserHeaderFfi,
            PayloadFfi,
            UserHeaderFfi,
        >,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(port_factory),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<PortFactoryRequestResponseUnion>
pub struct iox2_port_factory_request_response_storage_t {
    internal: [u8; 1656], // magic number obtained with size_of::<Option<PortFactoryRequestResponseUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(PortFactoryRequestResponseUnion)]
pub struct iox2_port_factory_request_response_t {
    service_type: iox2_service_type_e,
    value: iox2_port_factory_request_response_storage_t,
    deleter: fn(*mut iox2_port_factory_request_response_t),
}

impl iox2_port_factory_request_response_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: PortFactoryRequestResponseUnion,
        deleter: fn(*mut iox2_port_factory_request_response_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_port_factory_request_response_h_t;
/// The owning handle for `iox2_port_factory_request_response_t`. Passing the handle to an function transfers the ownership.
pub type iox2_port_factory_request_response_h = *mut iox2_port_factory_request_response_h_t;
/// The non-owning handle for `iox2_port_factory_request_response_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_port_factory_request_response_h_ref = *const iox2_port_factory_request_response_h;

impl AssertNonNullHandle for iox2_port_factory_request_response_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_port_factory_request_response_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_port_factory_request_response_h {
    type Target = *mut iox2_port_factory_request_response_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_port_factory_request_response_h_ref {
    type Target = *mut iox2_port_factory_request_response_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}
// END type definition
