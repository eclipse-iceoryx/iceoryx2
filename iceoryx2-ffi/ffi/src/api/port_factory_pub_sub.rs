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

use crate::api::{
    iox2_port_factory_publisher_builder_h, iox2_port_factory_publisher_builder_t,
    iox2_port_factory_subscriber_builder_h, iox2_port_factory_subscriber_builder_t,
    iox2_service_type_e, HandleToType, NoUserHeaderFfi, PayloadFfi,
    PortFactoryPublisherBuilderUnion, PortFactorySubscriberBuilderUnion,
};
use crate::iox2_static_config_publish_subscribe_t;

use iceoryx2::prelude::*;
use iceoryx2::service::port_factory::publish_subscribe::PortFactory;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::mem::ManuallyDrop;

// BEGIN types definition

pub(super) union PortFactoryPubSubUnion {
    ipc: ManuallyDrop<PortFactory<ipc::Service, PayloadFfi, NoUserHeaderFfi>>,
    local: ManuallyDrop<PortFactory<local::Service, PayloadFfi, NoUserHeaderFfi>>,
}

impl PortFactoryPubSubUnion {
    pub(super) fn new_ipc(
        port_factory: PortFactory<ipc::Service, PayloadFfi, NoUserHeaderFfi>,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(port_factory),
        }
    }
    pub(super) fn new_local(
        port_factory: PortFactory<local::Service, PayloadFfi, NoUserHeaderFfi>,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(port_factory),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<PortFactoryPubSubUnion>
pub struct iox2_port_factory_pub_sub_storage_t {
    internal: [u8; 1656], // magic number obtained with size_of::<Option<PortFactoryPubSubUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(PortFactoryPubSubUnion)]
pub struct iox2_port_factory_pub_sub_t {
    service_type: iox2_service_type_e,
    value: iox2_port_factory_pub_sub_storage_t,
    deleter: fn(*mut iox2_port_factory_pub_sub_t),
}

impl iox2_port_factory_pub_sub_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: PortFactoryPubSubUnion,
        deleter: fn(*mut iox2_port_factory_pub_sub_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_port_factory_pub_sub_h_t;
/// The owning handle for `iox2_port_factory_pub_sub_t`. Passing the handle to an function transfers the ownership.
pub type iox2_port_factory_pub_sub_h = *mut iox2_port_factory_pub_sub_h_t;

pub struct iox2_port_factory_pub_sub_ref_h_t;
/// The non-owning handle for `iox2_port_factory_pub_sub_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_port_factory_pub_sub_ref_h = *mut iox2_port_factory_pub_sub_ref_h_t;

impl HandleToType for iox2_port_factory_pub_sub_h {
    type Target = *mut iox2_port_factory_pub_sub_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_port_factory_pub_sub_ref_h {
    type Target = *mut iox2_port_factory_pub_sub_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

// END type definition

// BEGIN C API

/// This function casts an owning [`iox2_port_factory_pub_sub_h`] into a non-owning [`iox2_port_factory_pub_sub_ref_h`]
///
/// # Arguments
///
/// * `port_factory_handle` obtained by [`iox2_service_builder_pub_sub_open`](crate::iox2_service_builder_pub_sub_open) or
///   [`iox2_service_builder_pub_sub_open_or_create`](crate::iox2_service_builder_pub_sub_open_or_create)
///
/// Returns a [`iox2_port_factory_pub_sub_ref_h`]
///
/// # Safety
///
/// * The `port_factory_handle` must be a valid handle.
/// * The `port_factory_handle` is still valid after the call to this function.
#[no_mangle]
pub unsafe extern "C" fn iox2_cast_port_factory_pub_sub_ref_h(
    port_factory_handle: iox2_port_factory_pub_sub_h,
) -> iox2_port_factory_pub_sub_ref_h {
    debug_assert!(!port_factory_handle.is_null());

    (*port_factory_handle.as_type()).as_ref_handle() as *mut _ as _
}

/// Instantiates a [`iox2_port_factory_publisher_builder_h`] to build a publisher.
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_pub_sub_ref_h`] obtained
///   by e.g. [`iox2_service_builder_pub_sub_open_or_create`](crate::iox2_service_builder_pub_sub_open_or_create)
///   and casted by [`iox2_cast_port_factory_pub_sub_ref_h`]
/// * `publisher_builder_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_port_factory_publisher_builder_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
///
/// Returns the `iox2_port_factory_publisher_builder_h` handle for the publisher builder.
///
/// # Safety
///
/// * The `port_factory_handle` is still valid after the return of this function and can be use in another function call.
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_pub_sub_publisher_builder(
    port_factory_handle: iox2_port_factory_pub_sub_ref_h,
    publisher_builder_struct_ptr: *mut iox2_port_factory_publisher_builder_t,
) -> iox2_port_factory_publisher_builder_h {
    debug_assert!(!port_factory_handle.is_null());

    let mut publisher_builder_struct_ptr = publisher_builder_struct_ptr;
    fn no_op(_: *mut iox2_port_factory_publisher_builder_t) {}
    let mut deleter: fn(*mut iox2_port_factory_publisher_builder_t) = no_op;
    if publisher_builder_struct_ptr.is_null() {
        publisher_builder_struct_ptr = iox2_port_factory_publisher_builder_t::alloc();
        deleter = iox2_port_factory_publisher_builder_t::dealloc;
    }
    debug_assert!(!publisher_builder_struct_ptr.is_null());

    let port_factory = &mut *port_factory_handle.as_type();
    match port_factory.service_type {
        iox2_service_type_e::IPC => {
            let publisher_builder = port_factory.value.as_ref().ipc.publisher_builder();
            (*publisher_builder_struct_ptr).init(
                port_factory.service_type,
                PortFactoryPublisherBuilderUnion::new_ipc(publisher_builder),
                deleter,
            );
        }
        iox2_service_type_e::LOCAL => {
            let publisher_builder = port_factory.value.as_ref().local.publisher_builder();
            (*publisher_builder_struct_ptr).init(
                port_factory.service_type,
                PortFactoryPublisherBuilderUnion::new_local(publisher_builder),
                deleter,
            );
        }
    };

    (*publisher_builder_struct_ptr).as_handle()
}

/// Instantiates a [`iox2_port_factory_subscriber_builder_h`] to build a subscriber.
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_pub_sub_ref_h`] obtained
///   by e.g. [`iox2_service_builder_pub_sub_open_or_create`](crate::iox2_service_builder_pub_sub_open_or_create)
///   and casted by [`iox2_cast_port_factory_pub_sub_ref_h`]
/// * `subscriber_builder_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_port_factory_subscriber_builder_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
///
/// Returns the [`iox2_port_factory_subscriber_builder_h`] handle for the subscriber builder.
///
/// # Safety
///
/// * The `port_factory_handle` is still valid after the return of this function and can be use in another function call.
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_pub_sub_subscriber_builder(
    port_factory_handle: iox2_port_factory_pub_sub_ref_h,
    subscriber_builder_struct_ptr: *mut iox2_port_factory_subscriber_builder_t,
) -> iox2_port_factory_subscriber_builder_h {
    debug_assert!(!port_factory_handle.is_null());

    let mut subscriber_builder_struct_ptr = subscriber_builder_struct_ptr;
    fn no_op(_: *mut iox2_port_factory_subscriber_builder_t) {}
    let mut deleter: fn(*mut iox2_port_factory_subscriber_builder_t) = no_op;
    if subscriber_builder_struct_ptr.is_null() {
        subscriber_builder_struct_ptr = iox2_port_factory_subscriber_builder_t::alloc();
        deleter = iox2_port_factory_subscriber_builder_t::dealloc;
    }
    debug_assert!(!subscriber_builder_struct_ptr.is_null());

    let port_factory = &mut *port_factory_handle.as_type();
    match port_factory.service_type {
        iox2_service_type_e::IPC => {
            let subscriber_builder = port_factory.value.as_ref().ipc.subscriber_builder();
            (*subscriber_builder_struct_ptr).init(
                port_factory.service_type,
                PortFactorySubscriberBuilderUnion::new_ipc(subscriber_builder),
                deleter,
            );
        }
        iox2_service_type_e::LOCAL => {
            let subscriber_builder = port_factory.value.as_ref().local.subscriber_builder();
            (*subscriber_builder_struct_ptr).init(
                port_factory.service_type,
                PortFactorySubscriberBuilderUnion::new_local(subscriber_builder),
                deleter,
            );
        }
    };

    (*subscriber_builder_struct_ptr).as_handle()
}

/// Set the values int the provided [`iox2_static_config_publish_subscribe_t`] pointer.
///
/// # Safety
///
/// * The `_handle` must be valid and obtained by [`iox2_service_builder_pub_sub_open`](crate::iox2_service_builder_pub_sub_open) or
///   [`iox2_service_builder_pub_sub_open_or_create`](crate::iox2_service_builder_pub_sub_open_or_create)!
/// * The `static_config` must be a valid pointer and non-null.
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_pub_sub_static_config(
    port_factory_handle: iox2_port_factory_pub_sub_ref_h,
    static_config: *mut iox2_static_config_publish_subscribe_t,
) {
    debug_assert!(!port_factory_handle.is_null());
    debug_assert!(!static_config.is_null());

    let port_factory = &mut *port_factory_handle.as_type();

    use iceoryx2::prelude::PortFactory;
    let config = match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory.value.as_ref().ipc.static_config(),
        iox2_service_type_e::LOCAL => port_factory.value.as_ref().local.static_config(),
    };

    *static_config = config.into();
}

/// This function needs to be called to destroy the port factory!
///
/// # Arguments
///
/// * `port_factory_handle` - A valid [`iox2_port_factory_pub_sub_h`]
///
/// # Safety
///
/// * The `port_factory_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_port_factory_pub_sub_t`] can be re-used with a call to
///   [`iox2_service_builder_pub_sub_open_or_create`](crate::iox2_service_builder_pub_sub_open_or_create) or
///   [`iox2_service_builder_pub_sub_open`](crate::iox2_service_builder_pub_sub_open)!
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_pub_sub_drop(
    port_factory_handle: iox2_port_factory_pub_sub_h,
) {
    debug_assert!(!port_factory_handle.is_null());

    let port_factory = &mut *port_factory_handle.as_type();

    match port_factory.service_type {
        iox2_service_type_e::IPC => {
            ManuallyDrop::drop(&mut port_factory.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            ManuallyDrop::drop(&mut port_factory.value.as_mut().local);
        }
    }
    (port_factory.deleter)(port_factory);
}

// END C API
