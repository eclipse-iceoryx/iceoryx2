// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use crate::api::{AssertNonNullHandle, HandleToType, IOX2_OK, IntoCInt, iox2_service_type_e};
use crate::iox2_file_descriptor_ptr;

use iceoryx2::user_trigger::{
    UserTrigger, UserTriggerCreateError, UserTriggerNotifyError, UserTriggerResetError,
};
use iceoryx2_bb_elementary_traits::AsCStr;
use iceoryx2_bb_posix::file_descriptor::{FileDescriptor, FileDescriptorBased};
use iceoryx2_ffi_macros::CStrRepr;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::ffi::{c_char, c_int};
use core::mem::ManuallyDrop;

// BEGIN types definition

#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_user_trigger_create_error_e {
    INSUFFICIENT_RESOURCES = IOX2_OK as isize + 1,
    INTERNAL_ERROR,
}

impl IntoCInt for UserTriggerCreateError {
    fn into_c_int(self) -> c_int {
        (match self {
            UserTriggerCreateError::InsufficientResources => {
                iox2_user_trigger_create_error_e::INSUFFICIENT_RESOURCES
            }
            UserTriggerCreateError::InternalError => {
                iox2_user_trigger_create_error_e::INTERNAL_ERROR
            }
        }) as c_int
    }
}

#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_user_trigger_notify_error_e {
    INTERNAL_ERROR = IOX2_OK as isize + 1,
}

impl IntoCInt for UserTriggerNotifyError {
    fn into_c_int(self) -> c_int {
        (match self {
            UserTriggerNotifyError::InternalError => {
                iox2_user_trigger_notify_error_e::INTERNAL_ERROR
            }
        }) as c_int
    }
}

#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_user_trigger_reset_error_e {
    INTERNAL_ERROR = IOX2_OK as isize + 1,
}

impl IntoCInt for UserTriggerResetError {
    fn into_c_int(self) -> c_int {
        (match self {
            UserTriggerResetError::InternalError => iox2_user_trigger_reset_error_e::INTERNAL_ERROR,
        }) as c_int
    }
}

pub(super) union UserTriggerUnion {
    ipc: ManuallyDrop<UserTrigger<crate::IpcService>>,
    local: ManuallyDrop<UserTrigger<crate::LocalService>>,
}

impl UserTriggerUnion {
    pub(super) fn new_ipc(user_trigger: UserTrigger<crate::IpcService>) -> Self {
        Self {
            ipc: ManuallyDrop::new(user_trigger),
        }
    }
    pub(super) fn new_local(user_trigger: UserTrigger<crate::LocalService>) -> Self {
        Self {
            local: ManuallyDrop::new(user_trigger),
        }
    }
}

#[repr(C)]
#[repr(align(4))] // alignment of Option<UserTriggerUnion>
pub struct iox2_user_trigger_storage_t {
    internal: [u8; 28], // magic number obtained with size_of::<Option<UserTriggerUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(UserTriggerUnion)]
pub struct iox2_user_trigger_t {
    service_type: iox2_service_type_e,
    value: iox2_user_trigger_storage_t,
    deleter: fn(*mut iox2_user_trigger_t),
}

impl iox2_user_trigger_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: UserTriggerUnion,
        deleter: fn(*mut iox2_user_trigger_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_user_trigger_h_t;
/// The owning handle for `iox2_user_trigger_t`. Passing the handle to a function transfers the ownership.
pub type iox2_user_trigger_h = *mut iox2_user_trigger_h_t;
/// The non-owning handle for `iox2_user_trigger_t`. Passing the handle to a function does not transfer the ownership.
pub type iox2_user_trigger_h_ref = *const iox2_user_trigger_h;

impl AssertNonNullHandle for iox2_user_trigger_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_user_trigger_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_user_trigger_h {
    type Target = *mut iox2_user_trigger_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_user_trigger_h_ref {
    type Target = *mut iox2_user_trigger_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition

// BEGIN C API

/// Returns a string representation of the provided [`iox2_user_trigger_create_error_e`] error code.
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_user_trigger_create_error_string(
    error: iox2_user_trigger_create_error_e,
) -> *const c_char {
    error.as_const_cstr().as_ptr() as *const c_char
}

/// Returns a string representation of the provided [`iox2_user_trigger_notify_error_e`] error code.
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_user_trigger_notify_error_string(
    error: iox2_user_trigger_notify_error_e,
) -> *const c_char {
    error.as_const_cstr().as_ptr() as *const c_char
}

/// Returns a string representation of the provided [`iox2_user_trigger_reset_error_e`] error code.
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_user_trigger_reset_error_string(
    error: iox2_user_trigger_reset_error_e,
) -> *const c_char {
    error.as_const_cstr().as_ptr() as *const c_char
}

/// This function needs to be called to destroy the user trigger!
///
/// # Arguments
///
/// * `user_trigger_handle` - A valid [`iox2_user_trigger_h`]
///
/// # Safety
///
/// * The `user_trigger_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_user_trigger_t`] can be re-used with a call to
///   [`iox2_node_create_user_trigger`](crate::iox2_node_create_user_trigger)!
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_user_trigger_drop(user_trigger_handle: iox2_user_trigger_h) {
    user_trigger_handle.assert_non_null();
    unsafe {
        let user_trigger = &mut *user_trigger_handle.as_type();

        match user_trigger.service_type {
            iox2_service_type_e::IPC => {
                ManuallyDrop::drop(&mut user_trigger.value.as_mut().ipc);
            }
            iox2_service_type_e::LOCAL => {
                ManuallyDrop::drop(&mut user_trigger.value.as_mut().local);
            }
        }
        (user_trigger.deleter)(user_trigger);
    }
}

/// Returns the underlying non-owning file descriptor of the [`iox2_user_trigger_h`]. It can be
/// attached to a WaitSet with [`iox2_waitset_attach_notification`](crate::iox2_waitset_attach_notification).
///
/// # Arguments
///
/// * `user_trigger_handle` - A valid [`iox2_user_trigger_h_ref`],
///
/// # Safety
///
/// * The `user_trigger_handle` must be a valid handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_user_trigger_get_file_descriptor(
    user_trigger_handle: iox2_user_trigger_h_ref,
) -> iox2_file_descriptor_ptr {
    user_trigger_handle.assert_non_null();
    unsafe {
        let user_trigger = &mut *user_trigger_handle.as_type();

        let fd = match user_trigger.service_type {
            iox2_service_type_e::IPC => user_trigger.value.as_ref().ipc.file_descriptor(),
            iox2_service_type_e::LOCAL => user_trigger.value.as_ref().local.file_descriptor(),
        };
        (fd as *const FileDescriptor).cast()
    }
}

/// Wakes up every WaitSet the user trigger is attached to, until it is reset with
/// [`iox2_user_trigger_reset()`].
/// On error it returns [`iox2_user_trigger_notify_error_e`].
///
/// # Arguments
///
/// * `user_trigger_handle` - A valid [`iox2_user_trigger_h_ref`],
///
/// # Safety
///
/// * The `user_trigger_handle` must be a valid handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_user_trigger_notify(
    user_trigger_handle: iox2_user_trigger_h_ref,
) -> c_int {
    user_trigger_handle.assert_non_null();
    unsafe {
        let user_trigger = &mut *user_trigger_handle.as_type();

        let result = match user_trigger.service_type {
            iox2_service_type_e::IPC => user_trigger.value.as_ref().ipc.notify(),
            iox2_service_type_e::LOCAL => user_trigger.value.as_ref().local.notify(),
        };

        match result {
            Ok(()) => IOX2_OK,
            Err(e) => e.into_c_int(),
        }
    }
}

/// Consumes all notifications of the user trigger and sets `was_notified` to true when it was
/// notified since the last reset, otherwise to false.
/// On error it returns [`iox2_user_trigger_reset_error_e`].
///
/// # Arguments
///
/// * `user_trigger_handle` - A valid [`iox2_user_trigger_h_ref`],
/// * `was_notified` - A valid pointer to a `bool`.
///
/// # Safety
///
/// * The `user_trigger_handle` must be a valid handle.
/// * `was_notified` must be a valid pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_user_trigger_reset(
    user_trigger_handle: iox2_user_trigger_h_ref,
    was_notified: *mut bool,
) -> c_int {
    user_trigger_handle.assert_non_null();
    debug_assert!(!was_notified.is_null());
    unsafe {
        let user_trigger = &mut *user_trigger_handle.as_type();

        let result = match user_trigger.service_type {
            iox2_service_type_e::IPC => user_trigger.value.as_ref().ipc.reset(),
            iox2_service_type_e::LOCAL => user_trigger.value.as_ref().local.reset(),
        };

        match result {
            Ok(v) => {
                *was_notified = v;
                IOX2_OK
            }
            Err(e) => e.into_c_int(),
        }
    }
}

// END C API
