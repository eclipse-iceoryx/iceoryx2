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

use crate::api::{iox2_service_type_e, HandleToType};

use iceoryx2::prelude::*;
use iceoryx2::service::builder::{
    event::Builder as ServiceBuilderEvent, publish_subscribe::Builder as ServiceBuilderPubSub,
    Builder as ServiceBuilderBase,
};
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::mem::ManuallyDrop;

// BEGIN types definition

type UserHeader = [u8;128];
type Payload = [u8];

pub(super) union ServiceBuilderUnionNested<S: Service> {
    base: ManuallyDrop<ServiceBuilderBase<S>>,
    event: ManuallyDrop<ServiceBuilderEvent<S>>,
    pub_sub: ManuallyDrop<ServiceBuilderPubSub<Payload, UserHeader, S>>,
}

pub(super) union ServiceBuilderUnion {
    ipc: ManuallyDrop<ServiceBuilderUnionNested<zero_copy::Service>>,
    local: ManuallyDrop<ServiceBuilderUnionNested<process_local::Service>>,
}

impl ServiceBuilderUnion {
    pub(super) fn new_ipc_base(service_builder: ServiceBuilderBase<zero_copy::Service>) -> Self {
        Self {
            ipc: ManuallyDrop::new(ServiceBuilderUnionNested::<zero_copy::Service> {
                base: ManuallyDrop::new(service_builder),
            }),
        }
    }

    pub(super) fn new_ipc_event(service_builder: ServiceBuilderEvent<zero_copy::Service>) -> Self {
        Self {
            ipc: ManuallyDrop::new(ServiceBuilderUnionNested::<zero_copy::Service> {
                event: ManuallyDrop::new(service_builder),
            }),
        }
    }

    pub(super) fn new_ipc_pub_sub(
        service_builder: ServiceBuilderPubSub<Payload, UserHeader, zero_copy::Service>,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(ServiceBuilderUnionNested::<zero_copy::Service> {
                pub_sub: ManuallyDrop::new(service_builder),
            }),
        }
    }

    pub(super) fn new_local_base(
        service_builder: ServiceBuilderBase<process_local::Service>,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(ServiceBuilderUnionNested::<process_local::Service> {
                base: ManuallyDrop::new(service_builder),
            }),
        }
    }

    pub(super) fn new_local_event(
        service_builder: ServiceBuilderEvent<process_local::Service>,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(ServiceBuilderUnionNested::<process_local::Service> {
                event: ManuallyDrop::new(service_builder),
            }),
        }
    }

    pub(super) fn new_local_pub_sub(
        service_builder: ServiceBuilderPubSub<Payload, UserHeader, process_local::Service>,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(ServiceBuilderUnionNested::<process_local::Service> {
                pub_sub: ManuallyDrop::new(service_builder),
            }),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<ServiceBuilderUnion>
pub struct iox2_service_builder_storage_t {
    internal: [u8; 360], // magic number obtained with size_of::<Option<ServiceBuilderUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(ServiceBuilderUnion)]
pub struct iox2_service_builder_t {
    service_type: iox2_service_type_e,
    // TODO messaging_type for event and pub_sub differentiation
    value: iox2_service_builder_storage_t,
    deleter: fn(*mut iox2_service_builder_t),
}

impl iox2_service_builder_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: ServiceBuilderUnion,
        deleter: fn(*mut iox2_service_builder_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_service_builder_h_t;
/// The owning handle for `iox2_service_builder_t`. Passing the handle to an function transfers the ownership.
pub type iox2_service_builder_h = *mut iox2_service_builder_h_t;

pub struct iox2_service_builder_ref_h_t;
/// The non-owning handle for `iox2_service_builder_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_service_builder_ref_h = *mut iox2_service_builder_ref_h_t;

impl HandleToType for iox2_service_builder_h {
    type Target = *mut iox2_service_builder_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_service_builder_ref_h {
    type Target = *mut iox2_service_builder_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

// END type definition

// BEGIN C API

#[no_mangle]
pub extern "C" fn iox2_service_builder_event() {
    todo!()
}

// END C API
