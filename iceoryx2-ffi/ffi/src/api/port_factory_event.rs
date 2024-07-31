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
    iox2_port_factory_listener_builder_h, iox2_port_factory_listener_builder_t,
    iox2_port_factory_notifier_builder_h, iox2_port_factory_notifier_builder_t,
    iox2_service_name_ptr, iox2_service_type_e, HandleToType, PortFactoryListenerBuilderUnion,
    PortFactoryNotifierBuilderUnion,
};

use iceoryx2::prelude::*;
use iceoryx2::service::port_factory::{event::PortFactory as PortFactoryEvent, PortFactory};
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::mem::ManuallyDrop;

use super::iox2_static_config_event_t;

// BEGIN types definition

pub(super) union PortFactoryEventUnion {
    ipc: ManuallyDrop<PortFactoryEvent<zero_copy::Service>>,
    local: ManuallyDrop<PortFactoryEvent<process_local::Service>>,
}

impl PortFactoryEventUnion {
    pub(super) fn new_ipc(port_factory: PortFactoryEvent<zero_copy::Service>) -> Self {
        Self {
            ipc: ManuallyDrop::new(port_factory),
        }
    }
    pub(super) fn new_local(port_factory: PortFactoryEvent<process_local::Service>) -> Self {
        Self {
            local: ManuallyDrop::new(port_factory),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<PortFactoryEventUnion>
pub struct iox2_port_factory_event_storage_t {
    internal: [u8; 1656], // magic number obtained with size_of::<Option<PortFactoryEventUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(PortFactoryEventUnion)]
pub struct iox2_port_factory_event_t {
    service_type: iox2_service_type_e,
    value: iox2_port_factory_event_storage_t,
    deleter: fn(*mut iox2_port_factory_event_t),
}

impl iox2_port_factory_event_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: PortFactoryEventUnion,
        deleter: fn(*mut iox2_port_factory_event_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_port_factory_event_h_t;
/// The owning handle for `iox2_port_factory_event_t`. Passing the handle to an function transfers the ownership.
pub type iox2_port_factory_event_h = *mut iox2_port_factory_event_h_t;

pub struct iox2_port_factory_event_ref_h_t;
/// The non-owning handle for `iox2_port_factory_event_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_port_factory_event_ref_h = *mut iox2_port_factory_event_ref_h_t;

impl HandleToType for iox2_port_factory_event_h {
    type Target = *mut iox2_port_factory_event_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_port_factory_event_ref_h {
    type Target = *mut iox2_port_factory_event_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

// END type definition

// BEGIN C API

/// This function casts an owning [`iox2_port_factory_event_h`] into a non-owning [`iox2_port_factory_event_ref_h`]
///
/// # Arguments
///
/// * `port_factory_handle` obtained by [`iox2_service_builder_event_open`](crate::iox2_service_builder_event_open) or
///   [`iox2_service_builder_event_open_or_create`](crate::iox2_service_builder_event_open_or_create)
///
/// Returns a [`iox2_port_factory_event_ref_h`]
///
/// # Safety
///
/// * The `port_factory_handle` must be a valid handle.
/// * The `port_factory_handle` is still valid after the call to this function.
#[no_mangle]
pub unsafe extern "C" fn iox2_cast_port_factory_event_ref_h(
    port_factory_handle: iox2_port_factory_event_h,
) -> iox2_port_factory_event_ref_h {
    debug_assert!(!port_factory_handle.is_null());

    (*port_factory_handle.as_type()).as_ref_handle() as *mut _ as _
}

/// Returns the [`iox2_service_name_ptr`], an immutable pointer to the service name.
///
/// # Safety
///
/// * The `_handle` must be valid and obtained by [`iox2_service_builder_event_open`](crate::iox2_service_builder_event_open) or
///   [`iox2_service_builder_event_open_or_create`](crate::iox2_service_builder_event_open_or_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_event_service_name(
    port_factory_handle: iox2_port_factory_event_ref_h,
) -> iox2_service_name_ptr {
    debug_assert!(!port_factory_handle.is_null());

    let port_factory = &mut *port_factory_handle.as_type();

    match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory.value.as_ref().ipc.name(),
        iox2_service_type_e::LOCAL => port_factory.value.as_ref().local.name(),
    }
}

/// Set the values int the provided [`iox2_static_config_event_t`] pointer.
///
/// # Safety
///
/// * The `_handle` must be valid and obtained by [`iox2_service_builder_event_open`](crate::iox2_service_builder_event_open) or
///   [`iox2_service_builder_event_open_or_create`](crate::iox2_service_builder_event_open_or_create)!
/// * The `static_config` must be a valid pointer and non-null.
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_event_static_config(
    port_factory_handle: iox2_port_factory_event_ref_h,
    static_config: *mut iox2_static_config_event_t,
) {
    debug_assert!(!port_factory_handle.is_null());
    debug_assert!(!static_config.is_null());

    let port_factory = &mut *port_factory_handle.as_type();

    let config = match port_factory.service_type {
        iox2_service_type_e::IPC => port_factory.value.as_ref().ipc.static_config(),
        iox2_service_type_e::LOCAL => port_factory.value.as_ref().local.static_config(),
    };

    *static_config = config.into();
}

// TODO [#210] add all the other setter methods

/// Instantiates a [`iox2_port_factory_notifier_builder_h`] to build a notifier.
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_event_ref_h`] obtained
///   by e.g. [`iox2_service_builder_event_open_or_create`](crate::iox2_service_builder_event_open_or_create)
///   and casted by [`iox2_cast_port_factory_event_ref_h`]
/// * `notifier_builder_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_port_factory_notifier_builder_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
///
/// Returns the `iox2_port_factory_notifier_builder_h` handle for the notifier builder.
///
/// # Safety
///
/// * The `port_factory_handle` is still valid after the return of this function and can be use in another function call.
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_event_notifier_builder(
    port_factory_handle: iox2_port_factory_event_ref_h,
    notifier_builder_struct_ptr: *mut iox2_port_factory_notifier_builder_t,
) -> iox2_port_factory_notifier_builder_h {
    debug_assert!(!port_factory_handle.is_null());

    let mut notifier_builder_struct_ptr = notifier_builder_struct_ptr;
    fn no_op(_: *mut iox2_port_factory_notifier_builder_t) {}
    let mut deleter: fn(*mut iox2_port_factory_notifier_builder_t) = no_op;
    if notifier_builder_struct_ptr.is_null() {
        notifier_builder_struct_ptr = iox2_port_factory_notifier_builder_t::alloc();
        deleter = iox2_port_factory_notifier_builder_t::dealloc;
    }
    debug_assert!(!notifier_builder_struct_ptr.is_null());

    let port_factory = &mut *port_factory_handle.as_type();
    match port_factory.service_type {
        iox2_service_type_e::IPC => {
            let notifier_builder = port_factory.value.as_ref().ipc.notifier_builder();
            (*notifier_builder_struct_ptr).init(
                port_factory.service_type,
                PortFactoryNotifierBuilderUnion::new_ipc(notifier_builder),
                deleter,
            );
        }
        iox2_service_type_e::LOCAL => {
            let notifier_builder = port_factory.value.as_ref().local.notifier_builder();
            (*notifier_builder_struct_ptr).init(
                port_factory.service_type,
                PortFactoryNotifierBuilderUnion::new_local(notifier_builder),
                deleter,
            );
        }
    };

    (*notifier_builder_struct_ptr).as_handle()
}

/// Instantiates a [`iox2_port_factory_listener_builder_h`] to build a listener.
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_event_ref_h`] obtained
///   by e.g. [`iox2_service_builder_event_open_or_create`](crate::iox2_service_builder_event_open_or_create)
///   and casted by [`iox2_cast_port_factory_event_ref_h`]
/// * `listener_builder_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_port_factory_listener_builder_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
///
/// Returns the [`iox2_port_factory_listener_builder_h`] handle for the listener builder.
///
/// # Safety
///
/// * The `port_factory_handle` is still valid after the return of this function and can be use in another function call.
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_event_listener_builder(
    port_factory_handle: iox2_port_factory_event_ref_h,
    listener_builder_struct_ptr: *mut iox2_port_factory_listener_builder_t,
) -> iox2_port_factory_listener_builder_h {
    debug_assert!(!port_factory_handle.is_null());

    let mut listener_builder_struct_ptr = listener_builder_struct_ptr;
    fn no_op(_: *mut iox2_port_factory_listener_builder_t) {}
    let mut deleter: fn(*mut iox2_port_factory_listener_builder_t) = no_op;
    if listener_builder_struct_ptr.is_null() {
        listener_builder_struct_ptr = iox2_port_factory_listener_builder_t::alloc();
        deleter = iox2_port_factory_listener_builder_t::dealloc;
    }
    debug_assert!(!listener_builder_struct_ptr.is_null());

    let port_factory = &mut *port_factory_handle.as_type();
    match port_factory.service_type {
        iox2_service_type_e::IPC => {
            let listener_builder = port_factory.value.as_ref().ipc.listener_builder();
            (*listener_builder_struct_ptr).init(
                port_factory.service_type,
                PortFactoryListenerBuilderUnion::new_ipc(listener_builder),
                deleter,
            );
        }
        iox2_service_type_e::LOCAL => {
            let listener_builder = port_factory.value.as_ref().local.listener_builder();
            (*listener_builder_struct_ptr).init(
                port_factory.service_type,
                PortFactoryListenerBuilderUnion::new_local(listener_builder),
                deleter,
            );
        }
    };

    (*listener_builder_struct_ptr).as_handle()
}

/// This function needs to be called to destroy the port factory!
///
/// # Arguments
///
/// * `port_factory_handle` - A valid [`iox2_port_factory_event_h`]
///
/// # Safety
///
/// * The `port_factory_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_port_factory_event_t`] can be re-used with a call to
///   [`iox2_service_builder_event_open_or_create`](crate::iox2_service_builder_event_open_or_create) or
///   [`iox2_service_builder_event_open`](crate::iox2_service_builder_event_open)!
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_event_drop(
    port_factory_handle: iox2_port_factory_event_h,
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
