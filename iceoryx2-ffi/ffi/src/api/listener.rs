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

use crate::api::{iox2_service_type_e, HandleToType, IntoCInt};
use crate::{iox2_callback_context, iox2_event_id_t, IOX2_OK};

use iceoryx2::port::listener::Listener;
use iceoryx2::prelude::*;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_cal::event::ListenerWaitError;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::ffi::c_int;
use core::mem::ManuallyDrop;
use core::time::Duration;

// BEGIN types definition

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_listener_wait_error_e {
    CONTRACT_VIOLATION = IOX2_OK as isize + 1,
    INTERNAL_FAILURE,
    INTERRUPT_SIGNAL,
}

impl IntoCInt for ListenerWaitError {
    fn into_c_int(self) -> c_int {
        (match self {
            ListenerWaitError::ContractViolation => iox2_listener_wait_error_e::CONTRACT_VIOLATION,
            ListenerWaitError::InterruptSignal => iox2_listener_wait_error_e::INTERRUPT_SIGNAL,
            ListenerWaitError::InternalFailure => iox2_listener_wait_error_e::INTERNAL_FAILURE,
        }) as c_int
    }
}

pub(super) union ListenerUnion {
    ipc: ManuallyDrop<Listener<ipc::Service>>,
    local: ManuallyDrop<Listener<local::Service>>,
}

impl ListenerUnion {
    pub(super) fn new_ipc(listener: Listener<ipc::Service>) -> Self {
        Self {
            ipc: ManuallyDrop::new(listener),
        }
    }
    pub(super) fn new_local(listener: Listener<local::Service>) -> Self {
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

pub type iox2_listener_wait_all_callback =
    extern "C" fn(*const iox2_event_id_t, iox2_callback_context);

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

/// Tries to wait on the listener and calls the callback for every received event providing the
/// corresponding [`iox2_event_id_t`] pointer to the event.
/// On error it returns [`iox2_listener_wait_error_e`].
///
/// # Arguments
///
/// * `listener_handle` - A valid [`iox2_listener_ref_h`],
/// * `callback` - A valid callback with [`iox2_listener_wait_all_callback`} signature
/// * `callback_ctx` - An optional callback context [`iox2_callback_context`} to e.g. store information across callback iterations
///
/// # Safety
///
/// * The `listener_handle` must be a valid handle.
/// * The `callback` must be a valid function pointer.
#[no_mangle]
pub unsafe extern "C" fn iox2_listener_try_wait_all(
    listener_handle: iox2_listener_ref_h,
    callback: iox2_listener_wait_all_callback,
    callback_ctx: iox2_callback_context,
) -> c_int {
    debug_assert!(!listener_handle.is_null());

    let listener = &mut *listener_handle.as_type();

    let wait_result = match listener.service_type {
        iox2_service_type_e::IPC => listener.value.as_mut().ipc.try_wait_all(|event_id| {
            callback(&event_id.into(), callback_ctx);
        }),
        iox2_service_type_e::LOCAL => listener.value.as_mut().local.try_wait_all(|event_id| {
            callback(&event_id.into(), callback_ctx);
        }),
    };

    match wait_result {
        Ok(()) => IOX2_OK,
        Err(e) => e.into_c_int(),
    }
}

/// Blocks the listener until at least one event was received or the provided timeout has passed.
/// When an event was received then it calls the callback for
/// every received event providing the corresponding [`iox2_event_id_t`] pointer to the event.
/// On error it returns [`iox2_listener_wait_error_e`].
///
/// # Arguments
///
/// * `listener_handle` - A valid [`iox2_listener_ref_h`],
/// * `callback` - A valid callback with [`iox2_listener_wait_all_callback`} signature
/// * `callback_ctx` - An optional callback context [`iox2_callback_context`} to e.g. store information across callback iterations
///
/// # Safety
///
/// * The `listener_handle` must be a valid handle.
/// * The `callback` must be a valid function pointer.
#[no_mangle]
pub unsafe extern "C" fn iox2_listener_timed_wait_all(
    listener_handle: iox2_listener_ref_h,
    callback: iox2_listener_wait_all_callback,
    callback_ctx: iox2_callback_context,
    seconds: u64,
    nanoseconds: u32,
) -> c_int {
    debug_assert!(!listener_handle.is_null());

    let listener = &mut *listener_handle.as_type();
    let timeout = Duration::from_secs(seconds) + Duration::from_nanos(nanoseconds as u64);

    let wait_result = match listener.service_type {
        iox2_service_type_e::IPC => listener.value.as_mut().ipc.timed_wait_all(
            |event_id| {
                callback(&event_id.into(), callback_ctx);
            },
            timeout,
        ),
        iox2_service_type_e::LOCAL => listener.value.as_mut().local.timed_wait_all(
            |event_id| {
                callback(&event_id.into(), callback_ctx);
            },
            timeout,
        ),
    };

    match wait_result {
        Ok(()) => IOX2_OK,
        Err(e) => e.into_c_int(),
    }
}

/// Blocks the listener until at least one event was received and then calls the callback for
/// every received event providing the corresponding [`iox2_event_id_t`] pointer to the event.
/// On error it returns [`iox2_listener_wait_error_e`].
///
/// # Arguments
///
/// * `listener_handle` - A valid [`iox2_listener_ref_h`],
/// * `callback` - A valid callback with [`iox2_listener_wait_all_callback`} signature
/// * `callback_ctx` - An optional callback context [`iox2_callback_context`} to e.g. store information across callback iterations
///
/// # Safety
///
/// * The `listener_handle` must be a valid handle.
/// * The `callback` must be a valid function pointer.
#[no_mangle]
pub unsafe extern "C" fn iox2_listener_blocking_wait_all(
    listener_handle: iox2_listener_ref_h,
    callback: iox2_listener_wait_all_callback,
    callback_ctx: iox2_callback_context,
) -> c_int {
    debug_assert!(!listener_handle.is_null());

    let listener = &mut *listener_handle.as_type();

    let wait_result = match listener.service_type {
        iox2_service_type_e::IPC => listener.value.as_mut().ipc.blocking_wait_all(|event_id| {
            callback(&event_id.into(), callback_ctx);
        }),
        iox2_service_type_e::LOCAL => listener.value.as_mut().local.blocking_wait_all(|event_id| {
            callback(&event_id.into(), callback_ctx);
        }),
    };

    match wait_result {
        Ok(()) => IOX2_OK,
        Err(e) => e.into_c_int(),
    }
}

/// Tries to wait on the listener. If there is no event id present it returns immediately and sets
/// the out parameter `has_received_one` to false. Otherwise, it sets the `event_id` out parameter
/// and `has_received_one` to true.
/// On error it returns [`iox2_listener_wait_error_e`].
///
/// # Arguments
///
/// * `listener_handle` - A valid [`iox2_listener_ref_h`],
/// * `event_id` - A pointer to an [`iox2_event_id_t`] to store the received id.
/// * `has_received_one` - A pointer to a [`bool`] that signals if an event id was received or not
///
/// # Safety
///
/// * All input arguments must be non-null.
#[no_mangle]
pub unsafe extern "C" fn iox2_listener_try_wait_one(
    listener_handle: iox2_listener_ref_h,
    event_id: *mut iox2_event_id_t,
    has_received_one: *mut bool,
) -> c_int {
    debug_assert!(!listener_handle.is_null());
    debug_assert!(!event_id.is_null());
    debug_assert!(!has_received_one.is_null());

    let listener = &mut *listener_handle.as_type();

    let wait_result = match listener.service_type {
        iox2_service_type_e::IPC => listener.value.as_mut().ipc.try_wait_one(),
        iox2_service_type_e::LOCAL => listener.value.as_mut().local.try_wait_one(),
    };

    *has_received_one = false;

    match wait_result {
        Ok(Some(e)) => {
            *event_id = e.into();
            *has_received_one = true;
        }
        Ok(None) => (),
        Err(error) => {
            return error.into_c_int();
        }
    }

    IOX2_OK
}

/// Blocks on the listener until an event id was received or the provided timeout has passed.
/// When no event id was received and the
/// function was interrupted by a signal, `has_received_one` is set to false.
/// Otherwise, it sets the `event_id` out parameter and `has_received_one` to true.
/// On error it returns [`iox2_listener_wait_error_e`].
///
/// # Arguments
///
/// * `listener_handle` - A valid [`iox2_listener_ref_h`],
/// * `event_id` - A pointer to an [`iox2_event_id_t`] to store the received id.
/// * `has_received_one` - A pointer to a [`bool`] that signals if an event id was received or not
/// * `seconds` - The timeout seconds part
/// * `nanoseconds` - The timeout nanoseconds part
///
/// # Safety
///
/// * All input arguments must be non-null.
#[no_mangle]
pub unsafe extern "C" fn iox2_listener_timed_wait_one(
    listener_handle: iox2_listener_ref_h,
    event_id: *mut iox2_event_id_t,
    has_received_one: *mut bool,
    seconds: u64,
    nanoseconds: u32,
) -> c_int {
    debug_assert!(!listener_handle.is_null());
    debug_assert!(!event_id.is_null());
    debug_assert!(!has_received_one.is_null());

    let listener = &mut *listener_handle.as_type();
    *has_received_one = false;

    let timeout = Duration::from_secs(seconds) + Duration::from_nanos(nanoseconds as u64);

    let wait_result = match listener.service_type {
        iox2_service_type_e::IPC => listener.value.as_mut().ipc.timed_wait_one(timeout),
        iox2_service_type_e::LOCAL => listener.value.as_mut().local.timed_wait_one(timeout),
    };

    match wait_result {
        Ok(Some(e)) => {
            *event_id = e.into();
            *has_received_one = true;
        }
        Ok(None) => (),
        Err(error) => {
            return error.into_c_int();
        }
    }

    IOX2_OK
}

/// Blocks on the listener until an event id was received. When no event id was received and the
/// function was interrupted by a signal, `has_received_one` is set to false.
/// Otherwise, it sets the `event_id` out parameter and `has_received_one` to true.
/// On error it returns [`iox2_listener_wait_error_e`].
///
/// # Arguments
///
/// * `listener_handle` - A valid [`iox2_listener_ref_h`],
/// * `event_id` - A pointer to an [`iox2_event_id_t`] to store the received id.
/// * `has_received_one` - A pointer to a [`bool`] that signals if an event id was received or not
///
/// # Safety
///
/// * All input arguments must be non-null.
#[no_mangle]
pub unsafe extern "C" fn iox2_listener_blocking_wait_one(
    listener_handle: iox2_listener_ref_h,
    event_id: *mut iox2_event_id_t,
    has_received_one: *mut bool,
) -> c_int {
    debug_assert!(!listener_handle.is_null());
    debug_assert!(!event_id.is_null());
    debug_assert!(!has_received_one.is_null());

    let listener = &mut *listener_handle.as_type();
    *has_received_one = false;

    let wait_result = match listener.service_type {
        iox2_service_type_e::IPC => listener.value.as_mut().ipc.blocking_wait_one(),
        iox2_service_type_e::LOCAL => listener.value.as_mut().local.blocking_wait_one(),
    };

    match wait_result {
        Ok(Some(e)) => {
            *event_id = e.into();
            *has_received_one = true;
        }
        Ok(None) => (),
        Err(error) => {
            return error.into_c_int();
        }
    }

    IOX2_OK
}

// END C API
