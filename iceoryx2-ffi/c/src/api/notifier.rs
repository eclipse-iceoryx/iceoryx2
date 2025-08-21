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
    c_size_t, iox2_event_id_t, iox2_service_type_e, iox2_unique_notifier_id_h,
    iox2_unique_notifier_id_t, AssertNonNullHandle, HandleToType, IntoCInt, IOX2_OK,
};

use iceoryx2::port::notifier::{Notifier, NotifierNotifyError};
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_bb_elementary_traits::AsCStr;
use iceoryx2_ffi_macros::iceoryx2_ffi;
use iceoryx2_ffi_macros::CStrRepr;

use core::ffi::{c_char, c_int};
use core::mem::ManuallyDrop;

// BEGIN types definition

#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_notifier_notify_error_e {
    EVENT_ID_OUT_OF_BOUNDS = IOX2_OK as isize + 1,
    MISSED_DEADLINE,
    UNABLE_TO_ACQUIRE_ELAPSED_TIME,
}

impl IntoCInt for NotifierNotifyError {
    fn into_c_int(self) -> c_int {
        (match self {
            NotifierNotifyError::EventIdOutOfBounds => {
                iox2_notifier_notify_error_e::EVENT_ID_OUT_OF_BOUNDS
            }
            NotifierNotifyError::MissedDeadline => iox2_notifier_notify_error_e::MISSED_DEADLINE,
            NotifierNotifyError::UnableToAcquireElapsedTime => {
                iox2_notifier_notify_error_e::UNABLE_TO_ACQUIRE_ELAPSED_TIME
            }
        }) as c_int
    }
}

pub(super) union NotifierUnion {
    ipc: ManuallyDrop<Notifier<crate::IpcService>>,
    local: ManuallyDrop<Notifier<crate::LocalService>>,
}

impl NotifierUnion {
    pub(super) fn new_ipc(notifier: Notifier<crate::IpcService>) -> Self {
        Self {
            ipc: ManuallyDrop::new(notifier),
        }
    }
    pub(super) fn new_local(notifier: Notifier<crate::LocalService>) -> Self {
        Self {
            local: ManuallyDrop::new(notifier),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<NotifierUnion>
pub struct iox2_notifier_storage_t {
    internal: [u8; 1656], // magic number obtained with size_of::<Option<NotifierUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(NotifierUnion)]
pub struct iox2_notifier_t {
    service_type: iox2_service_type_e,
    value: iox2_notifier_storage_t,
    deleter: fn(*mut iox2_notifier_t),
}

impl iox2_notifier_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: NotifierUnion,
        deleter: fn(*mut iox2_notifier_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_notifier_h_t;
/// The owning handle for `iox2_notifier_t`. Passing the handle to an function transfers the ownership.
pub type iox2_notifier_h = *mut iox2_notifier_h_t;
/// The non-owning handle for `iox2_notifier_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_notifier_h_ref = *const iox2_notifier_h;

impl AssertNonNullHandle for iox2_notifier_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_notifier_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_notifier_h {
    type Target = *mut iox2_notifier_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_notifier_h_ref {
    type Target = *mut iox2_notifier_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition

// BEGIN C API

/// Returns a string literal describing the provided [`iox2_notifier_notify_error_e`].
///
/// # Arguments
///
/// * `error` - The error value for which a description should be returned
///
/// # Returns
///
/// A pointer to a null-terminated string containing the error message.
/// The string is stored in the .rodata section of the binary.
///
/// # Safety
///
/// The returned pointer must not be modified or freed and is valid as long as the program runs.
#[no_mangle]
pub unsafe extern "C" fn iox2_notifier_notify_error_string(
    error: iox2_notifier_notify_error_e,
) -> *const c_char {
    error.as_const_cstr().as_ptr() as *const c_char
}

/// Returns the unique port id of the notifier.
///
/// # Safety
///
/// * `notifier_handle` is valid, non-null and was obtained via [`iox2_port_factory_notifier_builder_create`](crate::iox2_port_factory_notifier_builder_create).
/// * `id_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_unique_notifier_id_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
/// * `id_handle_ptr` valid pointer to a [`iox2_unique_notifier_id_h`].
#[no_mangle]
pub unsafe extern "C" fn iox2_notifier_id(
    notifier_handle: iox2_notifier_h_ref,
    id_struct_ptr: *mut iox2_unique_notifier_id_t,
    id_handle_ptr: *mut iox2_unique_notifier_id_h,
) {
    notifier_handle.assert_non_null();
    debug_assert!(!id_handle_ptr.is_null());

    fn no_op(_: *mut iox2_unique_notifier_id_t) {}
    let mut deleter: fn(*mut iox2_unique_notifier_id_t) = no_op;
    let mut storage_ptr = id_struct_ptr;
    if id_struct_ptr.is_null() {
        deleter = iox2_unique_notifier_id_t::dealloc;
        storage_ptr = iox2_unique_notifier_id_t::alloc();
    }
    debug_assert!(!storage_ptr.is_null());

    let notifier = &mut *notifier_handle.as_type();

    let id = match notifier.service_type {
        iox2_service_type_e::IPC => notifier.value.as_mut().ipc.id(),
        iox2_service_type_e::LOCAL => notifier.value.as_mut().local.id(),
    };

    (*storage_ptr).init(id, deleter);
    *id_handle_ptr = (*storage_ptr).as_handle();
}

/// Returns the deadline of the notifier's service. If there is a deadline set, the provided
/// arguments `seconds` and `nanoseconds` will be set `true` is returned. Otherwise, false is
/// returned and nothing is set.
///
/// # Safety
///
/// * `notifier_handle` is valid, non-null and was obtained via [`iox2_port_factory_listener_builder_create`](crate::iox2_port_factory_listener_builder_create)
/// * `seconds` is pointing to a valid memory location and non-null
/// * `nanoseconds` is pointing to a valid memory location and non-null
#[no_mangle]
pub unsafe extern "C" fn iox2_notifier_deadline(
    notifier_handle: iox2_notifier_h_ref,
    seconds: *mut u64,
    nanoseconds: *mut u32,
) -> bool {
    notifier_handle.assert_non_null();
    debug_assert!(!seconds.is_null());
    debug_assert!(!nanoseconds.is_null());

    let notifier = &mut *notifier_handle.as_type();

    let deadline = match notifier.service_type {
        iox2_service_type_e::IPC => notifier.value.as_mut().ipc.deadline(),
        iox2_service_type_e::LOCAL => notifier.value.as_mut().local.deadline(),
    };

    deadline
        .map(|v| {
            *seconds = v.as_secs();
            *nanoseconds = v.subsec_nanos();
        })
        .is_some()
}

/// Notifies all [`iox2_listener_h`](crate::iox2_listener_h) connected to the service
/// with the default event id provided on creation.
///
/// # Arguments
///
/// * notifier_handle -  Must be a valid [`iox2_notifier_h_ref`]
///   obtained by [`iox2_port_factory_notifier_builder_create`](crate::iox2_port_factory_notifier_builder_create)
/// * number_of_notified_listener_ptr - Must be either a NULL pointer or a pointer to a `size_t` to store the number of notified listener
///
/// Returns IOX2_OK on success, an [`iox2_notifier_notify_error_e`] otherwise.
///
/// # Safety
///
/// `notifier_handle` must be a valid handle and is still valid after the return of this function and can be use in another function call.
#[no_mangle]
pub unsafe extern "C" fn iox2_notifier_notify(
    notifier_handle: iox2_notifier_h_ref,
    number_of_notified_listener_ptr: *mut c_size_t,
) -> c_int {
    notifier_handle.assert_non_null();

    let notifier = &mut *notifier_handle.as_type();

    let notify_result = match notifier.service_type {
        iox2_service_type_e::IPC => notifier.value.as_mut().ipc.notify(),
        iox2_service_type_e::LOCAL => notifier.value.as_mut().local.notify(),
    };

    match notify_result {
        Ok(count) => {
            if !number_of_notified_listener_ptr.is_null() {
                *number_of_notified_listener_ptr = count;
            }
        }
        Err(error) => {
            return error.into_c_int();
        }
    }

    IOX2_OK
}

/// Notifies all [`iox2_listener_h`](crate::iox2_listener_h) connected to the service
/// with the custom event id.
///
/// # Arguments
///
/// * notifier_handle -  Must be a valid [`iox2_notifier_h_ref`]
///   obtained by [`iox2_port_factory_notifier_builder_create`](crate::iox2_port_factory_notifier_builder_create)
/// * custom_event_id_ptr - Must be a pointer to an initialized [`iox2_event_id_t`](crate::iox2_event_id_t)
/// * number_of_notified_listener_ptr - Must be either a NULL pointer or a pointer to a `size_t` to store the number of notified listener
///
/// Returns IOX2_OK on success, an [`iox2_notifier_notify_error_e`] otherwise.
///
/// # Safety
///
/// `notifier_handle` must be a valid handle and is still valid after the return of this function and can be use in another function call.
/// `custom_event_id_ptr` must not be a NULL pointer.
#[no_mangle]
pub unsafe extern "C" fn iox2_notifier_notify_with_custom_event_id(
    notifier_handle: iox2_notifier_h_ref,
    custom_event_id_ptr: *const iox2_event_id_t,
    number_of_notified_listener_ptr: *mut c_size_t,
) -> c_int {
    notifier_handle.assert_non_null();
    debug_assert!(!custom_event_id_ptr.is_null());

    let event_id = (*custom_event_id_ptr).into();

    let notifier = &mut *notifier_handle.as_type();
    let notify_result = match notifier.service_type {
        iox2_service_type_e::IPC => notifier
            .value
            .as_mut()
            .ipc
            .notify_with_custom_event_id(event_id),
        iox2_service_type_e::LOCAL => notifier
            .value
            .as_mut()
            .local
            .notify_with_custom_event_id(event_id),
    };

    match notify_result {
        Ok(count) => {
            if !number_of_notified_listener_ptr.is_null() {
                *number_of_notified_listener_ptr = count;
            }
        }
        Err(error) => {
            return error.into_c_int();
        }
    }

    IOX2_OK
}

/// This function needs to be called to destroy the notifier!
///
/// # Arguments
///
/// * `notifier_handle` - A valid [`iox2_notifier_h`]
///
/// # Safety
///
/// * The `notifier_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_notifier_t`] can be re-used with a call to
///   [`iox2_port_factory_notifier_builder_create`](crate::iox2_port_factory_notifier_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_notifier_drop(notifier_handle: iox2_notifier_h) {
    debug_assert!(!notifier_handle.is_null());

    let notifier = &mut *notifier_handle.as_type();

    match notifier.service_type {
        iox2_service_type_e::IPC => {
            ManuallyDrop::drop(&mut notifier.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            ManuallyDrop::drop(&mut notifier.value.as_mut().local);
        }
    }
    (notifier.deleter)(notifier);
}

// END C API
