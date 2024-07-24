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
use iceoryx2::service::port_factory::subscriber::PortFactorySubscriber;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::mem::ManuallyDrop;

// BEGIN types definition

pub(super) union PortFactorySubscriberBuilderUnion {
    ipc: ManuallyDrop<
        PortFactorySubscriber<'static, zero_copy::Service, PayloadFfi, NoUserHeaderFfi>,
    >,
    local: ManuallyDrop<
        PortFactorySubscriber<'static, process_local::Service, PayloadFfi, NoUserHeaderFfi>,
    >,
}

impl PortFactorySubscriberBuilderUnion {
    pub(super) fn new_ipc(
        port_factory: PortFactorySubscriber<
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
        port_factory: PortFactorySubscriber<
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
#[repr(align(16))] // alignment of Option<PortFactorySubscriberBuilderUnion>
pub struct iox2_port_factory_subscriber_builder_storage_t {
    internal: [u8; 112], // magic number obtained with size_of::<Option<PortFactorySubscriberBuilderUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(PortFactorySubscriberBuilderUnion)]
pub struct iox2_port_factory_subscriber_builder_t {
    service_type: iox2_service_type_e,
    value: iox2_port_factory_subscriber_builder_storage_t,
    deleter: fn(*mut iox2_port_factory_subscriber_builder_t),
}

impl iox2_port_factory_subscriber_builder_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: PortFactorySubscriberBuilderUnion,
        deleter: fn(*mut iox2_port_factory_subscriber_builder_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_port_factory_subscriber_builder_h_t;
/// The owning handle for `iox2_port_factory_subscriber_builder_t`. Passing the handle to an function transfers the ownership.
pub type iox2_port_factory_subscriber_builder_h = *mut iox2_port_factory_subscriber_builder_h_t;

pub struct iox2_port_factory_subscriber_builder_ref_h_t;
/// The non-owning handle for `iox2_port_factory_subscriber_builder_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_port_factory_subscriber_builder_ref_h =
    *mut iox2_port_factory_subscriber_builder_ref_h_t;

impl HandleToType for iox2_port_factory_subscriber_builder_h {
    type Target = *mut iox2_port_factory_subscriber_builder_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_port_factory_subscriber_builder_ref_h {
    type Target = *mut iox2_port_factory_subscriber_builder_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

// END type definition
