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
use iceoryx2::service::port_factory::publisher::PortFactoryPublisher;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::mem::ManuallyDrop;

// BEGIN types definition

pub(super) union PortFactoryPublisherBuilderUnion {
    ipc: ManuallyDrop<
        PortFactoryPublisher<'static, zero_copy::Service, PayloadFfi, NoUserHeaderFfi>,
    >,
    local: ManuallyDrop<
        PortFactoryPublisher<'static, process_local::Service, PayloadFfi, NoUserHeaderFfi>,
    >,
}

impl PortFactoryPublisherBuilderUnion {
    pub(super) fn new_ipc(
        port_factory: PortFactoryPublisher<
            'static,
            zero_copy::Service,
            PayloadFfi,
            NoUserHeaderFfi,
        >,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(port_factory),
        }
    }
    pub(super) fn new_local(
        port_factory: PortFactoryPublisher<
            'static,
            process_local::Service,
            PayloadFfi,
            NoUserHeaderFfi,
        >,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(port_factory),
        }
    }
}

#[repr(C)]
#[repr(align(16))] // alignment of Option<PortFactoryPublisherBuilderUnion>
pub struct iox2_port_factory_publisher_builder_storage_t {
    internal: [u8; 128], // magic number obtained with size_of::<Option<PortFactoryPublisherBuilderUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(PortFactoryPublisherBuilderUnion)]
pub struct iox2_port_factory_publisher_builder_t {
    service_type: iox2_service_type_e,
    value: iox2_port_factory_publisher_builder_storage_t,
    deleter: fn(*mut iox2_port_factory_publisher_builder_t),
}

impl iox2_port_factory_publisher_builder_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: PortFactoryPublisherBuilderUnion,
        deleter: fn(*mut iox2_port_factory_publisher_builder_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_port_factory_publisher_builder_h_t;
/// The owning handle for `iox2_port_factory_publisher_builder_t`. Passing the handle to an function transfers the ownership.
pub type iox2_port_factory_publisher_builder_h = *mut iox2_port_factory_publisher_builder_h_t;

pub struct iox2_port_factory_publisher_builder_ref_h_t;
/// The non-owning handle for `iox2_port_factory_publisher_builder_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_port_factory_publisher_builder_ref_h =
    *mut iox2_port_factory_publisher_builder_ref_h_t;

impl HandleToType for iox2_port_factory_publisher_builder_h {
    type Target = *mut iox2_port_factory_publisher_builder_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_port_factory_publisher_builder_ref_h {
    type Target = *mut iox2_port_factory_publisher_builder_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

// END type definition
