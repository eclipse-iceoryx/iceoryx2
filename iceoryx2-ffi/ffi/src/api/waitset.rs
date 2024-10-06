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

use std::{ffi::c_int, mem::ManuallyDrop};

use crate::{c_size_t, iox2_attachment_id_h, iox2_callback_context, iox2_service_type_e, IOX2_OK};

use super::{AssertNonNullHandle, HandleToType, IntoCInt};
use iceoryx2::{
    port::waitset::{
        WaitSet, WaitSetAttachmentError, WaitSetCreateError, WaitSetRunError, WaitSetRunResult,
    },
    service::{ipc, local},
};
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

// BEGIN types definition

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_waitset_run_error_e {
    INSUFFICIENT_PERMISSIONS = IOX2_OK as isize + 1,
    INTERNAL_ERROR,
    NO_ATTACHMENTS,
    TERMINATION_REQUEST,
    INTERRUPT,
}

impl IntoCInt for WaitSetRunError {
    fn into_c_int(self) -> c_int {
        (match self {
            WaitSetRunError::InsufficientPermissions => {
                iox2_waitset_run_error_e::INSUFFICIENT_PERMISSIONS
            }
            WaitSetRunError::InternalError => iox2_waitset_run_error_e::INTERNAL_ERROR,
            WaitSetRunError::NoAttachments => iox2_waitset_run_error_e::NO_ATTACHMENTS,
            WaitSetRunError::TerminationRequest => iox2_waitset_run_error_e::TERMINATION_REQUEST,
            WaitSetRunError::Interrupt => iox2_waitset_run_error_e::INTERRUPT,
        }) as c_int
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_waitset_run_result_e {
    TERMINATION_REQUEST = IOX2_OK as isize + 1,
    INTERRUPT,
    STOP_REQUEST,
}

impl IntoCInt for WaitSetRunResult {
    fn into_c_int(self) -> c_int {
        (match self {
            WaitSetRunResult::TerminationRequest => iox2_waitset_run_result_e::TERMINATION_REQUEST,
            WaitSetRunResult::Interrupt => iox2_waitset_run_result_e::INTERRUPT,
            WaitSetRunResult::StopRequest => iox2_waitset_run_result_e::STOP_REQUEST,
        }) as c_int
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_waitset_attachment_error_e {
    INSUFFICIENT_CAPACITY = IOX2_OK as isize + 1,
    ALREADY_ATTACHED,
    INTERNAL_ERROR,
}

impl IntoCInt for WaitSetAttachmentError {
    fn into_c_int(self) -> c_int {
        (match self {
            WaitSetAttachmentError::InsufficientCapacity => {
                iox2_waitset_attachment_error_e::INSUFFICIENT_CAPACITY
            }
            WaitSetAttachmentError::AlreadyAttached => {
                iox2_waitset_attachment_error_e::ALREADY_ATTACHED
            }
            WaitSetAttachmentError::InternalError => {
                iox2_waitset_attachment_error_e::INTERNAL_ERROR
            }
        }) as c_int
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_waitset_create_error_e {
    INTERNAL_ERROR = IOX2_OK as isize + 1,
}

impl IntoCInt for WaitSetCreateError {
    fn into_c_int(self) -> c_int {
        (match self {
            WaitSetCreateError::InternalError => iox2_waitset_create_error_e::INTERNAL_ERROR,
        }) as c_int
    }
}

pub(super) union WaitSetUnion {
    ipc: ManuallyDrop<WaitSet<ipc::Service>>,
    local: ManuallyDrop<WaitSet<local::Service>>,
}

impl WaitSetUnion {
    pub(super) fn new_ipc(waitset: WaitSet<ipc::Service>) -> Self {
        Self {
            ipc: ManuallyDrop::new(waitset),
        }
    }

    pub(super) fn new_local(waitset: WaitSet<local::Service>) -> Self {
        Self {
            local: ManuallyDrop::new(waitset),
        }
    }
}

#[repr(C)]
#[repr(align(16))] // alignment of Option<WaitSetUnion>
pub struct iox2_waitset_storage_t {
    internal: [u8; 384], // magic number obtained with size_of::<Option<WaitSetUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(WaitSetUnion)]
pub struct iox2_waitset_t {
    service_type: iox2_service_type_e,
    value: iox2_waitset_storage_t,
    deleter: fn(*mut iox2_waitset_t),
}

impl iox2_waitset_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: WaitSetUnion,
        deleter: fn(*mut iox2_waitset_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_waitset_h_t;
/// The owning handle for `iox2_waitset_t`. Passing the handle to an function transfers the ownership.
pub type iox2_waitset_h = *mut iox2_waitset_h_t;
/// The non-owning handle for `iox2_waitset_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_waitset_h_ref = *const iox2_waitset_h;

impl AssertNonNullHandle for iox2_waitset_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_waitset_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_waitset_h {
    type Target = *mut iox2_waitset_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_waitset_h_ref {
    type Target = *mut iox2_waitset_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

pub type iox2_waitset_run_callback = extern "C" fn(iox2_attachment_id_h, iox2_callback_context);

// END type definition

// BEGIN C API
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_drop(handle: iox2_waitset_h) {
    handle.assert_non_null();

    let waitset = &mut *handle.as_type();

    match waitset.service_type {
        iox2_service_type_e::IPC => {
            ManuallyDrop::drop(&mut waitset.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            ManuallyDrop::drop(&mut waitset.value.as_mut().local);
        }
    }
    (waitset.deleter)(waitset);
}

#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_is_empty(handle: iox2_waitset_h_ref) -> bool {
    handle.assert_non_null();

    let waitset = &mut *handle.as_type();

    match waitset.service_type {
        iox2_service_type_e::IPC => waitset.value.as_ref().ipc.is_empty(),
        iox2_service_type_e::LOCAL => waitset.value.as_ref().local.is_empty(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_len(handle: iox2_waitset_h_ref) -> c_size_t {
    handle.assert_non_null();

    let waitset = &mut *handle.as_type();

    match waitset.service_type {
        iox2_service_type_e::IPC => waitset.value.as_ref().ipc.len(),
        iox2_service_type_e::LOCAL => waitset.value.as_ref().local.len(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_capacity(handle: iox2_waitset_h_ref) -> c_size_t {
    handle.assert_non_null();

    let waitset = &mut *handle.as_type();

    match waitset.service_type {
        iox2_service_type_e::IPC => waitset.value.as_ref().ipc.capacity(),
        iox2_service_type_e::LOCAL => waitset.value.as_ref().local.capacity(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_stop(handle: iox2_waitset_h_ref) {
    handle.assert_non_null();

    let waitset = &mut *handle.as_type();

    match waitset.service_type {
        iox2_service_type_e::IPC => waitset.value.as_ref().ipc.stop(),
        iox2_service_type_e::LOCAL => waitset.value.as_ref().local.stop(),
    }
}

/// Returns [`iox2_waitset_run_error_e`].
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_run_once(
    handle: iox2_waitset_h_ref,
    callback: iox2_waitset_run_callback,
) -> c_int {
    todo!()
}

/// Returns [`iox2_waitset_run_error_e`].
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_run(
    handle: iox2_waitset_h_ref,
    callback: iox2_waitset_run_callback,
    result: *mut iox2_waitset_run_result_e,
) -> c_int {
    todo!()
}

// END C API
