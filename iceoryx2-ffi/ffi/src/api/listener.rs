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

use iceoryx2::port::listener::Listener;
use iceoryx2::prelude::*;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::mem::ManuallyDrop;

// BEGIN types definition

pub(super) union ListenerUnion {
    ipc: ManuallyDrop<Listener<zero_copy::Service>>,
    local: ManuallyDrop<Listener<process_local::Service>>,
}

impl ListenerUnion {
    pub(super) fn new_ipc(listener: Listener<zero_copy::Service>) -> Self {
        Self {
            ipc: ManuallyDrop::new(listener),
        }
    }
    pub(super) fn new_local(listener: Listener<process_local::Service>) -> Self {
        Self {
            local: ManuallyDrop::new(listener),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<ListenerUnion>
pub struct iox2_listener_storage_t {
    internal: [u8; 1656], // magic number obtained with size_of::<Option<ListenerUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(ListenerUnion)]
pub struct iox2_listener_t {
    service_type: iox2_service_type_e,
    value: iox2_listener_storage_t,
    deleter: fn(*mut iox2_listener_t),
}

impl iox2_listener_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: ListenerUnion,
        deleter: fn(*mut iox2_listener_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_listener_h_t;
/// The owning handle for `iox2_listener_t`. Passing the handle to an function transfers the ownership.
pub type iox2_listener_h = *mut iox2_listener_h_t;

pub struct iox2_listener_ref_h_t;
/// The non-owning handle for `iox2_listener_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_listener_ref_h = *mut iox2_listener_ref_h_t;

impl HandleToType for iox2_listener_h {
    type Target = *mut iox2_listener_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_listener_ref_h {
    type Target = *mut iox2_listener_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

// END type definition

// BEGIN C API

/// This function casts an owning [`iox2_listener_h`] into a non-owning [`iox2_listener_ref_h`]
///
/// # Arguments
///
/// * `listener_handle` obtained by [`iox2_port_factory_listener_builder_create`](crate::iox2_port_factory_listener_builder_create)
///
/// Returns a [`iox2_listener_ref_h`]
///
/// # Safety
///
/// * The `listener_handle` must be a valid handle.
/// * The `listener_handle` is still valid after the call to this function.
#[no_mangle]
pub unsafe extern "C" fn iox2_cast_listener_ref_h(
    listener_handle: iox2_listener_h,
) -> iox2_listener_ref_h {
    debug_assert!(!listener_handle.is_null());

    (*listener_handle.as_type()).as_ref_handle() as *mut _ as _
}

/// This function needs to be called to destroy the listener!
///
/// # Arguments
///
/// * `listener_handle` - A valid [`iox2_listener_h`]
///
/// # Safety
///
/// * The `listener_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_listener_t`] can be re-used with a call to
///   [`iox2_port_factory_listener_builder_create`](crate::iox2_port_factory_listener_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_listener_drop(listener_handle: iox2_listener_h) {
    debug_assert!(!listener_handle.is_null());

    let listener = &mut *listener_handle.as_type();

    match listener.service_type {
        iox2_service_type_e::IPC => {
            ManuallyDrop::drop(&mut listener.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            ManuallyDrop::drop(&mut listener.value.as_mut().local);
        }
    }
    (listener.deleter)(listener);
}

// END C API
