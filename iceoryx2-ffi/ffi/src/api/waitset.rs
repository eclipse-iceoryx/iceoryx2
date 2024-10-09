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

use std::{ffi::c_int, mem::ManuallyDrop, time::Duration};

use crate::{
    c_size_t, iox2_callback_context, iox2_file_descriptor_ptr, iox2_service_type_e,
    iox2_waitset_attachment_id_h, iox2_waitset_attachment_id_t, iox2_waitset_guard_h,
    iox2_waitset_guard_t, AttachmentIdUnion, GuardUnion, IOX2_OK,
};

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
        Into::<iox2_waitset_run_result_e>::into(self) as c_int
    }
}

impl From<WaitSetRunResult> for iox2_waitset_run_result_e {
    fn from(value: WaitSetRunResult) -> Self {
        match value {
            WaitSetRunResult::TerminationRequest => iox2_waitset_run_result_e::TERMINATION_REQUEST,
            WaitSetRunResult::Interrupt => iox2_waitset_run_result_e::INTERRUPT,
            WaitSetRunResult::StopRequest => iox2_waitset_run_result_e::STOP_REQUEST,
        }
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

pub(crate) union WaitSetUnion {
    ipc: ManuallyDrop<WaitSet<ipc::Service>>,
    local: ManuallyDrop<WaitSet<local::Service>>,
}

impl WaitSetUnion {
    pub(crate) fn new_ipc(waitset: WaitSet<ipc::Service>) -> Self {
        Self {
            ipc: ManuallyDrop::new(waitset),
        }
    }

    pub(crate) fn new_local(waitset: WaitSet<local::Service>) -> Self {
        Self {
            local: ManuallyDrop::new(waitset),
        }
    }
}

#[repr(C)]
#[repr(align(16))] // alignment of Option<WaitSetUnion>
pub struct iox2_waitset_storage_t {
    internal: [u8; 784], // magic number obtained with size_of::<Option<WaitSetUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(WaitSetUnion)]
pub struct iox2_waitset_t {
    pub(crate) service_type: iox2_service_type_e,
    pub(crate) value: iox2_waitset_storage_t,
    pub(crate) deleter: fn(*mut iox2_waitset_t),
}

impl iox2_waitset_t {
    pub(crate) fn init(
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

pub type iox2_waitset_run_callback =
    extern "C" fn(iox2_waitset_attachment_id_h, iox2_callback_context);
// END type definition

// BEGIN C API
/// Drops a [`iox2_waitset_h`] and calls all corresponding cleanup functions.
///
/// # Safety
///
///  * `handle` must be valid and acquired with
///    [`iox2_waitset_builder_create()`](crate::iox2_waitset_builder_create())
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

/// Returns `true` if the [`iox2_waitset_h`] is empty, otherwise false.
///
/// # Safety
///
///  * `handle` must be valid and acquired with
///    [`iox2_waitset_builder_create()`](crate::iox2_waitset_builder_create())
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_is_empty(handle: iox2_waitset_h_ref) -> bool {
    handle.assert_non_null();

    let waitset = &mut *handle.as_type();

    match waitset.service_type {
        iox2_service_type_e::IPC => waitset.value.as_ref().ipc.is_empty(),
        iox2_service_type_e::LOCAL => waitset.value.as_ref().local.is_empty(),
    }
}

/// Returns the number of attachments of the [`iox2_waitset_h`].
///
/// # Safety
///
///  * `handle` must be valid and acquired with
///    [`iox2_waitset_builder_create()`](crate::iox2_waitset_builder_create())
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_len(handle: iox2_waitset_h_ref) -> c_size_t {
    handle.assert_non_null();

    let waitset = &mut *handle.as_type();

    match waitset.service_type {
        iox2_service_type_e::IPC => waitset.value.as_ref().ipc.len(),
        iox2_service_type_e::LOCAL => waitset.value.as_ref().local.len(),
    }
}

/// Returns the capacity of the [`iox2_waitset_h`].
///
/// # Safety
///
///  * `handle` must be valid and acquired with
///    [`iox2_waitset_builder_create()`](crate::iox2_waitset_builder_create())
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_capacity(handle: iox2_waitset_h_ref) -> c_size_t {
    handle.assert_non_null();

    let waitset = &mut *handle.as_type();

    match waitset.service_type {
        iox2_service_type_e::IPC => waitset.value.as_ref().ipc.capacity(),
        iox2_service_type_e::LOCAL => waitset.value.as_ref().local.capacity(),
    }
}

/// Stops the current [`iox2_waitset_wait_and_process()`] operation. Any [`iox2_waitset_wait_and_process()`]
/// call after this call is not affected and the user needs to call
/// [`iox2_waitset_stop()`] again.
///
/// # Safety
///
///  * `handle` must be valid and acquired with
///    [`iox2_waitset_builder_create()`](crate::iox2_waitset_builder_create())
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_stop(handle: iox2_waitset_h_ref) {
    handle.assert_non_null();

    let waitset = &mut *handle.as_type();

    match waitset.service_type {
        iox2_service_type_e::IPC => waitset.value.as_ref().ipc.stop(),
        iox2_service_type_e::LOCAL => waitset.value.as_ref().local.stop(),
    }
}

/// Attaches a provided [`iox2_file_descriptor_ptr`] as notification to the
/// [`iox2_waitset_h`]. As soon as the attachment receives data, the WaitSet
/// wakes up in [`iox2_waitset_wait_and_process()`] and informs the user.
///
/// With [`iox2_waitset_attachment_id_has_event_from()`](crate::iox2_waitset_attachment_id_has_event_from())
/// the origin of the event can be determined from its corresponding
/// [`iox2_waitset_guard_h`].
///
/// # Return
///
/// `IOX2_OK` on success, otherwise [`iox2_waitset_attachment_error_e`].
///
/// # Safety
///
///  * `handle` must be valid and acquired with
///    [`iox2_waitset_builder_create()`](crate::iox2_waitset_builder_create())
///  * `guard_struct_ptr` must be either pointing to a valid uninitialized memory
///    position or `null`
///  * `guard_handle_ptr` must be pointing to valid uninitialized memory.
///  * `guard_handle_ptr` must be released with [`iox2_waitset_guard_drop()`](crate::iox2_waitset_guard_drop()).
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_attach_notification(
    handle: iox2_waitset_h_ref,
    fd: iox2_file_descriptor_ptr,
    guard_struct_ptr: *mut iox2_waitset_guard_t,
    guard_handle_ptr: *mut iox2_waitset_guard_h,
) -> c_int {
    handle.assert_non_null();
    debug_assert!(!guard_handle_ptr.is_null());

    let waitset = &mut *handle.as_type();

    let mut guard_struct_ptr = guard_struct_ptr;
    fn no_op(_: *mut iox2_waitset_guard_t) {}
    let mut deleter: fn(*mut iox2_waitset_guard_t) = no_op;
    let mut alloc_memory = || {
        if guard_struct_ptr.is_null() {
            guard_struct_ptr = iox2_waitset_guard_t::alloc();
            deleter = iox2_waitset_guard_t::dealloc;
        }
        debug_assert!(!guard_struct_ptr.is_null());
    };

    match waitset.service_type {
        iox2_service_type_e::IPC => match waitset.value.as_ref().ipc.attach_notification(&*fd) {
            Ok(guard) => {
                alloc_memory();

                (*guard_struct_ptr).init(waitset.service_type, GuardUnion::new_ipc(guard), deleter);
            }
            Err(e) => {
                return e.into_c_int();
            }
        },
        iox2_service_type_e::LOCAL => {
            match waitset.value.as_ref().local.attach_notification(&*fd) {
                Ok(guard) => {
                    alloc_memory();
                    (*guard_struct_ptr).init(
                        waitset.service_type,
                        GuardUnion::new_local(guard),
                        deleter,
                    );
                }
                Err(e) => {
                    return e.into_c_int();
                }
            }
        }
    }

    *guard_handle_ptr = (*guard_struct_ptr).as_handle();

    IOX2_OK
}

/// Attaches a provided [`iox2_file_descriptor_ptr`] as deadline to the
/// [`iox2_waitset_h`]. As soon as the attachment receives data or the deadline
/// was missed, the WaitSet wakes up in [`iox2_waitset_wait_and_process()`] and informs the user.
///
/// With [`iox2_waitset_attachment_id_has_event_from()`](crate::iox2_waitset_attachment_id_has_event_from())
/// the origin of the event can be determined from its corresponding
/// [`iox2_waitset_guard_h`].
/// If the deadline was hit the function
/// [`iox2_waitset_attachment_id_has_missed_deadline()`](crate::iox2_waitset_attachment_id_has_missed_deadline())
/// can be used to identify it.
///
/// # Return
///
/// `IOX2_OK` on success, otherwise [`iox2_waitset_attachment_error_e`].
///
/// # Safety
///
///  * `handle` must be valid and acquired with
///    [`iox2_waitset_builder_create()`](crate::iox2_waitset_builder_create())
///  * `guard_struct_ptr` must be either pointing to a valid uninitialized memory
///    position or `null`
///  * `guard_handle_ptr` must be pointing to valid uninitialized memory.
///  * `guard_handle_ptr` must be released with [`iox2_waitset_guard_drop()`](crate::iox2_waitset_guard_drop()).
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_attach_deadline(
    handle: iox2_waitset_h_ref,
    fd: iox2_file_descriptor_ptr,
    seconds: u64,
    nanoseconds: u32,
    guard_struct_ptr: *mut iox2_waitset_guard_t,
    guard_handle_ptr: *mut iox2_waitset_guard_h,
) -> c_int {
    handle.assert_non_null();
    debug_assert!(!guard_handle_ptr.is_null());

    let waitset = &mut *handle.as_type();
    let interval = Duration::from_secs(seconds) + Duration::from_nanos(nanoseconds as _);

    let mut guard_struct_ptr = guard_struct_ptr;
    fn no_op(_: *mut iox2_waitset_guard_t) {}
    let mut deleter: fn(*mut iox2_waitset_guard_t) = no_op;
    let mut alloc_memory = || {
        if guard_struct_ptr.is_null() {
            guard_struct_ptr = iox2_waitset_guard_t::alloc();
            deleter = iox2_waitset_guard_t::dealloc;
        }
        debug_assert!(!guard_struct_ptr.is_null());
    };

    match waitset.service_type {
        iox2_service_type_e::IPC => {
            match waitset.value.as_ref().ipc.attach_deadline(&*fd, interval) {
                Ok(guard) => {
                    alloc_memory();

                    (*guard_struct_ptr).init(
                        waitset.service_type,
                        GuardUnion::new_ipc(guard),
                        deleter,
                    );
                }
                Err(e) => {
                    return e.into_c_int();
                }
            }
        }
        iox2_service_type_e::LOCAL => {
            match waitset.value.as_ref().local.attach_deadline(&*fd, interval) {
                Ok(guard) => {
                    alloc_memory();

                    (*guard_struct_ptr).init(
                        waitset.service_type,
                        GuardUnion::new_local(guard),
                        deleter,
                    );
                }
                Err(e) => {
                    return e.into_c_int();
                }
            }
        }
    }

    *guard_handle_ptr = (*guard_struct_ptr).as_handle();

    IOX2_OK
}

/// Attaches an interval to the [`iox2_waitset_h`]. As soon as the interval has passed
/// the WaitSet wakes up in [`iox2_waitset_wait_and_process()`] and informs the user.
///
/// With [`iox2_waitset_attachment_id_has_event_from()`](crate::iox2_waitset_attachment_id_has_event_from())
/// the origin of the event can be determined from its corresponding
/// [`iox2_waitset_guard_h`].
///
/// # Return
///
/// `IOX2_OK` on success, otherwise [`iox2_waitset_attachment_error_e`].
///
/// # Safety
///
///  * `handle` must be valid and acquired with
///    [`iox2_waitset_builder_create()`](crate::iox2_waitset_builder_create())
///  * `guard_struct_ptr` must be either pointing to a valid uninitialized memory
///    position or `null`
///  * `guard_handle_ptr` must be pointing to valid uninitialized memory.
///  * `guard_handle_ptr` must be released with [`iox2_waitset_guard_drop()`](crate::iox2_waitset_guard_drop()).
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_attach_interval(
    handle: iox2_waitset_h_ref,
    seconds: u64,
    nanoseconds: u32,
    guard_struct_ptr: *mut iox2_waitset_guard_t,
    guard_handle_ptr: *mut iox2_waitset_guard_h,
) -> c_int {
    handle.assert_non_null();
    debug_assert!(!guard_handle_ptr.is_null());

    let waitset = &mut *handle.as_type();
    let interval = Duration::from_secs(seconds) + Duration::from_nanos(nanoseconds as _);

    let mut guard_struct_ptr = guard_struct_ptr;
    fn no_op(_: *mut iox2_waitset_guard_t) {}
    let mut deleter: fn(*mut iox2_waitset_guard_t) = no_op;
    let mut alloc_memory = || {
        if guard_struct_ptr.is_null() {
            guard_struct_ptr = iox2_waitset_guard_t::alloc();
            deleter = iox2_waitset_guard_t::dealloc;
        }
        debug_assert!(!guard_struct_ptr.is_null());
    };

    match waitset.service_type {
        iox2_service_type_e::IPC => match waitset.value.as_ref().ipc.attach_interval(interval) {
            Ok(guard) => {
                alloc_memory();

                (*guard_struct_ptr).init(waitset.service_type, GuardUnion::new_ipc(guard), deleter);
            }
            Err(e) => {
                return e.into_c_int();
            }
        },
        iox2_service_type_e::LOCAL => {
            match waitset.value.as_ref().local.attach_interval(interval) {
                Ok(guard) => {
                    alloc_memory();

                    (*guard_struct_ptr).init(
                        waitset.service_type,
                        GuardUnion::new_local(guard),
                        deleter,
                    );
                }
                Err(e) => {
                    return e.into_c_int();
                }
            }
        }
    }

    *guard_handle_ptr = (*guard_struct_ptr).as_handle();

    IOX2_OK
}

/// Checks the [`iox2_waitset_h`] for new events once. The provided `callback` is called
/// for every events that was received and the corresponding owning [`iox2_waitset_attachment_id_h`]
/// is provided as input argument, as well as the `callback_ctx`.
///
/// With [`iox2_waitset_attachment_id_has_event_from()`](crate::iox2_waitset_attachment_id_has_event_from())
/// the origin of the event can be determined from its corresponding
/// [`iox2_waitset_guard_h`].
/// If the deadline was hit the function
/// [`iox2_waitset_attachment_id_has_missed_deadline()`](crate::iox2_waitset_attachment_id_has_missed_deadline())
/// can be used to identify it.
///
/// # Return
///
/// `IOX2_OK` on success, otherwise [`iox2_waitset_run_error_e`].
///
/// # Safety
///
///  * `handle` must be valid and acquired with
///    [`iox2_waitset_builder_create()`](crate::iox2_waitset_builder_create())
///  * the provided [`iox2_waitset_attachment_id_h`] in the callback must be released via
///    [`iox2_waitset_attachment_id_drop()`](crate::iox2_waitset_attachment_id_drop())
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_try_wait_and_process(
    handle: iox2_waitset_h_ref,
    callback: iox2_waitset_run_callback,
    callback_ctx: iox2_callback_context,
) -> c_int {
    handle.assert_non_null();

    let waitset = &mut *handle.as_type();

    let run_once_result = match waitset.service_type {
        iox2_service_type_e::IPC => {
            waitset
                .value
                .as_ref()
                .ipc
                .try_wait_and_process(|attachment_id| {
                    let attachment_id_ptr = iox2_waitset_attachment_id_t::alloc();
                    (*attachment_id_ptr).init(
                        waitset.service_type,
                        AttachmentIdUnion::new_ipc(attachment_id),
                        iox2_waitset_attachment_id_t::dealloc,
                    );
                    let attachment_id_handle_ptr = (*attachment_id_ptr).as_handle();
                    callback(attachment_id_handle_ptr, callback_ctx);
                })
        }
        iox2_service_type_e::LOCAL => {
            waitset
                .value
                .as_ref()
                .local
                .try_wait_and_process(|attachment_id| {
                    let attachment_id_ptr = iox2_waitset_attachment_id_t::alloc();
                    (*attachment_id_ptr).init(
                        waitset.service_type,
                        AttachmentIdUnion::new_local(attachment_id),
                        iox2_waitset_attachment_id_t::dealloc,
                    );
                    let attachment_id_handle_ptr = (*attachment_id_ptr).as_handle();
                    callback(attachment_id_handle_ptr, callback_ctx);
                })
        }
    };

    match run_once_result {
        Ok(()) => IOX2_OK,
        Err(e) => e.into_c_int(),
    }
}

/// Checks the [`iox2_waitset_h`] for new events in an infinite loop. The provided
/// `callback` is called for every events that was received and the corresponding
/// owning [`iox2_waitset_attachment_id_h`] is provided as input argument, as well as the
/// `callback_ctx`.
/// The infinite loop is interrupted either by a `SIGINT` or `SIGTERM` signal or
/// when the user has called [`iox2_waitset_stop()`].
///
/// With [`iox2_waitset_attachment_id_has_event_from()`](crate::iox2_waitset_attachment_id_has_event_from())
/// the origin of the event can be determined from its corresponding
/// [`iox2_waitset_guard_h`].
/// If the deadline was hit the function
/// [`iox2_waitset_attachment_id_has_missed_deadline()`](crate::iox2_waitset_attachment_id_has_missed_deadline())
/// can be used to identify it.
///
/// # Return
///
/// `IOX2_OK` on success, otherwise [`iox2_waitset_run_error_e`].
///
/// # Safety
///
///  * `handle` must be valid and acquired with
///    [`iox2_waitset_builder_create()`](crate::iox2_waitset_builder_create())
///  * the provided [`iox2_waitset_attachment_id_h`] in the callback must be released via
///    [`iox2_waitset_attachment_id_drop()`](crate::iox2_waitset_attachment_id_drop())
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_wait_and_process(
    handle: iox2_waitset_h_ref,
    callback: iox2_waitset_run_callback,
    callback_ctx: iox2_callback_context,
    result: *mut iox2_waitset_run_result_e,
) -> c_int {
    handle.assert_non_null();
    debug_assert!(!result.is_null());

    let waitset = &mut *handle.as_type();

    let run_result = match waitset.service_type {
        iox2_service_type_e::IPC => waitset
            .value
            .as_ref()
            .ipc
            .wait_and_process(|attachment_id| {
                let attachment_id_ptr = iox2_waitset_attachment_id_t::alloc();
                (*attachment_id_ptr).init(
                    waitset.service_type,
                    AttachmentIdUnion::new_ipc(attachment_id),
                    iox2_waitset_attachment_id_t::dealloc,
                );
                let attachment_id_handle_ptr = (*attachment_id_ptr).as_handle();
                callback(attachment_id_handle_ptr, callback_ctx);
            }),
        iox2_service_type_e::LOCAL => {
            waitset
                .value
                .as_ref()
                .local
                .wait_and_process(|attachment_id| {
                    let attachment_id_ptr = iox2_waitset_attachment_id_t::alloc();
                    (*attachment_id_ptr).init(
                        waitset.service_type,
                        AttachmentIdUnion::new_local(attachment_id),
                        iox2_waitset_attachment_id_t::dealloc,
                    );
                    let attachment_id_handle_ptr = (*attachment_id_ptr).as_handle();
                    callback(attachment_id_handle_ptr, callback_ctx);
                })
        }
    };

    match run_result {
        Ok(v) => {
            (*result) = v.into();
            IOX2_OK
        }
        Err(e) => e.into_c_int(),
    }
}

// END C API