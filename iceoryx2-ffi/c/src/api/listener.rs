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
#![allow(dead_code)]

use crate::api::{
    iox2_callback_context, iox2_event_id_t, iox2_service_type_e, iox2_unique_listener_id_h,
    iox2_unique_listener_id_t, AssertNonNullHandle, HandleToType, IntoCInt, IOX2_OK,
};
use crate::iox2_file_descriptor_ptr;

use iceoryx2::port::listener::Listener;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_bb_elementary_traits::AsCStr;
use iceoryx2_bb_posix::file_descriptor::{FileDescriptor, FileDescriptorBased};
use iceoryx2_cal::event::ListenerWaitError;
use iceoryx2_ffi_macros::iceoryx2_ffi;
use iceoryx2_ffi_macros::CStrRepr;

use core::ffi::{c_char, c_int};
use core::mem::ManuallyDrop;
use core::time::Duration;

use super::CFileDescriptor;

// BEGIN types definition

#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
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

trait AcquireFileDescriptor {
    fn acquire_file_descriptor(&self) -> Option<&FileDescriptor>;
}

struct AcquireFileDescriptorHopper<'a, T> {
    value: &'a T,
}

impl<'a, T> AcquireFileDescriptorHopper<'a, T> {
    fn new(value: &'a T) -> Self {
        Self { value }
    }
}

impl<T> AcquireFileDescriptor for AcquireFileDescriptorHopper<'_, T> {
    fn acquire_file_descriptor(&self) -> Option<&FileDescriptor> {
        None
    }
}

impl<T: FileDescriptorBased> AcquireFileDescriptorHopper<'_, T> {
    fn acquire_file_descriptor(&self) -> Option<&FileDescriptor> {
        Some(self.value.file_descriptor())
    }
}

pub(super) union ListenerUnion {
    ipc: ManuallyDrop<Listener<crate::IpcService>>,
    local: ManuallyDrop<Listener<crate::LocalService>>,
}

impl ListenerUnion {
    pub(super) fn new_ipc(listener: Listener<crate::IpcService>) -> Self {
        Self {
            ipc: ManuallyDrop::new(listener),
        }
    }
    pub(super) fn new_local(listener: Listener<crate::LocalService>) -> Self {
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
/// The non-owning handle for `iox2_listener_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_listener_h_ref = *const iox2_listener_h;

impl AssertNonNullHandle for iox2_listener_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_listener_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_listener_h {
    type Target = *mut iox2_listener_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_listener_h_ref {
    type Target = *mut iox2_listener_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

pub type iox2_listener_wait_all_callback =
    extern "C" fn(*const iox2_event_id_t, iox2_callback_context);

// END type definition

// BEGIN C API

/// Returns a string representation of the provided [`iox2_listener_wait_error_e`] error code.
///
/// # Arguments
///
/// * `error` - The error code that should be converted into a string
///
/// # Returns
///
/// A pointer to a null-terminated string containing the error message.
/// The string is stored in the .rodata section of the binary.
///
/// # Safety
///
/// * The returned pointer must not be modified or freed and is only valid as long as the program runs
#[no_mangle]
pub unsafe extern "C" fn iox2_listener_wait_error_string(
    error: iox2_listener_wait_error_e,
) -> *const c_char {
    error.as_const_cstr().as_ptr() as *const c_char
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
    listener_handle.assert_non_null();

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

/// Returns the underlying non-owning file descriptor of the [`iox2_listener_h`] if the
/// [`iox2_listener_h`] is file descriptor based, otherwise it returns NULL.
///
/// # Arguments
///
/// * `listener_handle` - A valid [`iox2_listener_h_ref`],
///
/// # Safety
///
/// * The `listener_handle` must be a valid handle.
#[no_mangle]
pub unsafe extern "C" fn iox2_listener_get_file_descriptor(
    listener_handle: iox2_listener_h_ref,
) -> iox2_file_descriptor_ptr {
    listener_handle.assert_non_null();

    let listener = &mut *listener_handle.as_type();

    match listener.service_type {
        iox2_service_type_e::IPC => {
            let hopper = AcquireFileDescriptorHopper::new(&*listener.value.as_ref().ipc);
            match hopper.acquire_file_descriptor() {
                Some(fd) => (fd as *const FileDescriptor).cast(),
                None => core::ptr::null::<CFileDescriptor>(),
            }
        }
        iox2_service_type_e::LOCAL => {
            let hopper = AcquireFileDescriptorHopper::new(&*listener.value.as_ref().local);
            match hopper.acquire_file_descriptor() {
                Some(fd) => (fd as *const FileDescriptor).cast(),
                None => core::ptr::null::<CFileDescriptor>(),
            }
        }
    }
}

/// Tries to wait on the listener and calls the callback for every received event providing the
/// corresponding [`iox2_event_id_t`] pointer to the event.
/// On error it returns [`iox2_listener_wait_error_e`].
///
/// # Arguments
///
/// * `listener_handle` - A valid [`iox2_listener_h_ref`],
/// * `callback` - A valid callback with [`iox2_listener_wait_all_callback`} signature
/// * `callback_ctx` - An optional callback context [`iox2_callback_context`} to e.g. store information across callback iterations
///
/// # Safety
///
/// * The `listener_handle` must be a valid handle.
/// * The `callback` must be a valid function pointer.
#[no_mangle]
pub unsafe extern "C" fn iox2_listener_try_wait_all(
    listener_handle: iox2_listener_h_ref,
    callback: iox2_listener_wait_all_callback,
    callback_ctx: iox2_callback_context,
) -> c_int {
    listener_handle.assert_non_null();

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
/// * `listener_handle` - A valid [`iox2_listener_h_ref`],
/// * `callback` - A valid callback with [`iox2_listener_wait_all_callback`} signature
/// * `callback_ctx` - An optional callback context [`iox2_callback_context`} to e.g. store information across callback iterations
///
/// # Safety
///
/// * The `listener_handle` must be a valid handle.
/// * The `callback` must be a valid function pointer.
#[no_mangle]
pub unsafe extern "C" fn iox2_listener_timed_wait_all(
    listener_handle: iox2_listener_h_ref,
    callback: iox2_listener_wait_all_callback,
    callback_ctx: iox2_callback_context,
    seconds: u64,
    nanoseconds: u32,
) -> c_int {
    listener_handle.assert_non_null();

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

/// Returns the unique port id of the listener.
///
/// # Arguments
///
/// * `handle` obtained by [`iox2_port_factory_listener_builder_create`](crate::iox2_port_factory_listener_builder_create)
/// * `id_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_unique_listener_id_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
/// * `id_handle_ptr` valid pointer to a [`iox2_unique_listener_id_h`].
///
/// # Safety
///
/// * `listener_handle` is valid, non-null and was obtained via [`iox2_port_factory_listener_builder_create`](crate::iox2_port_factory_listener_builder_create)
/// * `id` is valid and non-null
#[no_mangle]
pub unsafe extern "C" fn iox2_listener_id(
    listener_handle: iox2_listener_h_ref,
    id_struct_ptr: *mut iox2_unique_listener_id_t,
    id_handle_ptr: *mut iox2_unique_listener_id_h,
) {
    listener_handle.assert_non_null();
    debug_assert!(!id_handle_ptr.is_null());

    fn no_op(_: *mut iox2_unique_listener_id_t) {}
    let mut deleter: fn(*mut iox2_unique_listener_id_t) = no_op;
    let mut storage_ptr = id_struct_ptr;
    if id_struct_ptr.is_null() {
        deleter = iox2_unique_listener_id_t::dealloc;
        storage_ptr = iox2_unique_listener_id_t::alloc();
    }
    debug_assert!(!storage_ptr.is_null());

    let listener = &mut *listener_handle.as_type();

    let id = match listener.service_type {
        iox2_service_type_e::IPC => listener.value.as_mut().ipc.id(),
        iox2_service_type_e::LOCAL => listener.value.as_mut().local.id(),
    };

    (*storage_ptr).init(id, deleter);
    *id_handle_ptr = (*storage_ptr).as_handle();
}

/// Returns the deadline of the listener's service. If there is a deadline set, the provided
/// arguments `seconds` and `nanoseconds` will be set `true` is returned. Otherwise, false is
/// returned and nothing is set.
///
/// # Safety
///
/// * `listener_handle` is valid, non-null and was obtained via [`iox2_port_factory_listener_builder_create`](crate::iox2_port_factory_listener_builder_create)
/// * `seconds` is pointing to a valid memory location and non-null
/// * `nanoseconds` is pointing to a valid memory location and non-null
#[no_mangle]
pub unsafe extern "C" fn iox2_listener_deadline(
    listener_handle: iox2_listener_h_ref,
    seconds: *mut u64,
    nanoseconds: *mut u32,
) -> bool {
    listener_handle.assert_non_null();
    debug_assert!(!seconds.is_null());
    debug_assert!(!nanoseconds.is_null());

    let listener = &mut *listener_handle.as_type();

    let deadline = match listener.service_type {
        iox2_service_type_e::IPC => listener.value.as_mut().ipc.deadline(),
        iox2_service_type_e::LOCAL => listener.value.as_mut().local.deadline(),
    };

    deadline
        .map(|v| {
            *seconds = v.as_secs();
            *nanoseconds = v.subsec_nanos();
        })
        .is_some()
}

/// Blocks the listener until at least one event was received and then calls the callback for
/// every received event providing the corresponding [`iox2_event_id_t`] pointer to the event.
/// On error it returns [`iox2_listener_wait_error_e`].
///
/// # Arguments
///
/// * `listener_handle` - A valid [`iox2_listener_h_ref`],
/// * `callback` - A valid callback with [`iox2_listener_wait_all_callback`} signature
/// * `callback_ctx` - An optional callback context [`iox2_callback_context`} to e.g. store information across callback iterations
///
/// # Safety
///
/// * The `listener_handle` must be a valid handle.
/// * The `callback` must be a valid function pointer.
#[no_mangle]
pub unsafe extern "C" fn iox2_listener_blocking_wait_all(
    listener_handle: iox2_listener_h_ref,
    callback: iox2_listener_wait_all_callback,
    callback_ctx: iox2_callback_context,
) -> c_int {
    listener_handle.assert_non_null();

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
/// * `listener_handle` - A valid [`iox2_listener_h_ref`],
/// * `event_id` - A pointer to an [`iox2_event_id_t`] to store the received id.
/// * `has_received_one` - A pointer to a [`bool`] that signals if an event id was received or not
///
/// # Safety
///
/// * All input arguments must be non-null.
#[no_mangle]
pub unsafe extern "C" fn iox2_listener_try_wait_one(
    listener_handle: iox2_listener_h_ref,
    event_id: *mut iox2_event_id_t,
    has_received_one: *mut bool,
) -> c_int {
    listener_handle.assert_non_null();
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
/// * `listener_handle` - A valid [`iox2_listener_h_ref`],
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
    listener_handle: iox2_listener_h_ref,
    event_id: *mut iox2_event_id_t,
    has_received_one: *mut bool,
    seconds: u64,
    nanoseconds: u32,
) -> c_int {
    listener_handle.assert_non_null();
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
/// * `listener_handle` - A valid [`iox2_listener_h_ref`],
/// * `event_id` - A pointer to an [`iox2_event_id_t`] to store the received id.
/// * `has_received_one` - A pointer to a [`bool`] that signals if an event id was received or not
///
/// # Safety
///
/// * All input arguments must be non-null.
#[no_mangle]
pub unsafe extern "C" fn iox2_listener_blocking_wait_one(
    listener_handle: iox2_listener_h_ref,
    event_id: *mut iox2_event_id_t,
    has_received_one: *mut bool,
) -> c_int {
    listener_handle.assert_non_null();
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
