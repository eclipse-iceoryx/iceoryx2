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

use super::{iox2_service_type_e, PayloadFfi, UserHeaderFfi};
use super::{AssertNonNullHandle, HandleToType};
use iceoryx2::prelude::*;
use iceoryx2::service::port_factory::client::PortFactoryClient;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

// BEGIN types definition

pub(super) union PortFactoryClientBuilderUnion {
    ipc: ManuallyDrop<
        PortFactoryClient<
            'static,
            ipc::Service,
            PayloadFfi,
            UserHeaderFfi,
            PayloadFfi,
            UserHeaderFfi,
        >,
    >,
    local: ManuallyDrop<
        PortFactoryClient<
            'static,
            local::Service,
            PayloadFfi,
            UserHeaderFfi,
            PayloadFfi,
            UserHeaderFfi,
        >,
    >,
}

impl PortFactoryClientBuilderUnion {
    pub(super) fn new_ipc(
        port_factory: PortFactoryClient<
            'static,
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
        port_factory: PortFactoryClient<
            'static,
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
#[repr(align(16))] // alignment of Option<PortFactoryClientBuilderUnion>
pub struct iox2_port_factory_client_builder_storage_t {
    internal: [u8; 176], // magic number obtained with size_of::<Option<PortFactoryClientBuilderUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(PortFactoryClientBuilderUnion)]
pub struct iox2_port_factory_client_builder_t {
    service_type: iox2_service_type_e,
    value: iox2_port_factory_client_builder_storage_t,
    deleter: fn(*mut iox2_port_factory_client_builder_t),
}

impl iox2_port_factory_client_builder_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: PortFactoryClientBuilderUnion,
        deleter: fn(*mut iox2_port_factory_client_builder_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_port_factory_client_builder_h_t;
/// The owning handle for `iox2_port_factory_client_builder_t`. Passing the handle to an function transfers the ownership.
pub type iox2_port_factory_client_builder_h = *mut iox2_port_factory_client_builder_h_t;
/// The non-owning handle for `iox2_port_factory_client_builder_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_port_factory_client_builder_h_ref = *const iox2_port_factory_client_builder_h;

impl AssertNonNullHandle for iox2_port_factory_client_builder_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_port_factory_client_builder_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_port_factory_client_builder_h {
    type Target = *mut iox2_port_factory_client_builder_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_port_factory_client_builder_h_ref {
    type Target = *mut iox2_port_factory_client_builder_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition
