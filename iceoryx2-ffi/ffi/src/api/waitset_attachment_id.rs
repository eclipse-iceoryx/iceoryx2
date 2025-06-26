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

use crate::c_size_t;
use core::{ffi::c_char, mem::ManuallyDrop};

use iceoryx2::prelude::WaitSetAttachmentId;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use crate::{iox2_service_type_e, iox2_waitset_guard_h_ref};

use super::{AssertNonNullHandle, HandleToType};

// BEGIN types definition
pub(crate) union AttachmentIdUnion {
    ipc: ManuallyDrop<WaitSetAttachmentId<crate::IpcService>>,
    local: ManuallyDrop<WaitSetAttachmentId<crate::LocalService>>,
}

impl AttachmentIdUnion {
    pub(crate) fn new_ipc(attachment: WaitSetAttachmentId<crate::IpcService>) -> Self {
        Self {
            ipc: ManuallyDrop::new(attachment),
        }
    }

    pub(crate) fn new_local(attachment: WaitSetAttachmentId<crate::LocalService>) -> Self {
        Self {
            local: ManuallyDrop::new(attachment),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<AttachmentIdUnion>
pub struct iox2_waitset_attachment_id_storage_t {
    internal: [u8; 32], // magic number obtained with size_of::<Option<AttachmentIdUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(AttachmentIdUnion)]
pub struct iox2_waitset_attachment_id_t {
    service_type: iox2_service_type_e,
    value: iox2_waitset_attachment_id_storage_t,
    deleter: fn(*mut iox2_waitset_attachment_id_t),
}

impl iox2_waitset_attachment_id_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: AttachmentIdUnion,
        deleter: fn(*mut iox2_waitset_attachment_id_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_waitset_attachment_id_h_t;
/// The owning handle for `iox2_waitset_attachment_id_t`. Passing the handle to an function transfers the ownership.
pub type iox2_waitset_attachment_id_h = *mut iox2_waitset_attachment_id_h_t;
/// The non-owning handle for `iox2_waitset_attachment_id_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_waitset_attachment_id_h_ref = *const iox2_waitset_attachment_id_h;

impl AssertNonNullHandle for iox2_waitset_attachment_id_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_waitset_attachment_id_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_waitset_attachment_id_h {
    type Target = *mut iox2_waitset_attachment_id_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_waitset_attachment_id_h_ref {
    type Target = *mut iox2_waitset_attachment_id_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}
// END type definition

// BEGIN C API
/// Release an [`iox2_waitset_attachment_id_h`] that was acquired by calling either
/// * [`iox2_waitset_wait_and_process()`](crate::iox2_waitset_wait_and_process())
/// * [`iox2_waitset_wait_and_process_once()`](crate::iox2_waitset_wait_and_process_once())
///
/// # Safety
///  * `handle` must be valid and provided by the previously mentioned functions.
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_attachment_id_drop(handle: iox2_waitset_attachment_id_h) {
    handle.assert_non_null();

    let attachment_id = &mut *handle.as_type();

    match attachment_id.service_type {
        iox2_service_type_e::IPC => {
            ManuallyDrop::drop(&mut attachment_id.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            ManuallyDrop::drop(&mut attachment_id.value.as_mut().local);
        }
    }
    (attachment_id.deleter)(attachment_id);
}

/// Checks if two provided [`iox2_waitset_attachment_id_h_ref`] are semantically equal.
///
/// # Safety
///  * `lhs` must be valid and non-null.
///  * `rhs` must be valid and non-null.
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_attachment_id_equal(
    lhs: iox2_waitset_attachment_id_h_ref,
    rhs: iox2_waitset_attachment_id_h_ref,
) -> bool {
    lhs.assert_non_null();
    rhs.assert_non_null();

    let lhs_type = &mut *lhs.as_type();
    let rhs_type = &mut *rhs.as_type();

    if lhs_type.service_type != rhs_type.service_type {
        return false;
    }

    match lhs_type.service_type {
        iox2_service_type_e::IPC => *lhs_type.value.as_ref().ipc == *rhs_type.value.as_ref().ipc,
        iox2_service_type_e::LOCAL => {
            *lhs_type.value.as_ref().local == *rhs_type.value.as_ref().local
        }
    }
}

/// Checks the ordering of two provided [`iox2_waitset_attachment_id_h_ref`].
///
/// # Safety
///  * `lhs` must be valid and non-null.
///  * `rhs` must be valid and non-null.
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_attachment_id_less(
    lhs: iox2_waitset_attachment_id_h_ref,
    rhs: iox2_waitset_attachment_id_h_ref,
) -> bool {
    lhs.assert_non_null();
    rhs.assert_non_null();

    let lhs_type = &mut *lhs.as_type();
    let rhs_type = &mut *rhs.as_type();

    if lhs_type.service_type != rhs_type.service_type {
        return false;
    }

    match lhs_type.service_type {
        iox2_service_type_e::IPC => *lhs_type.value.as_ref().ipc < *rhs_type.value.as_ref().ipc,
        iox2_service_type_e::LOCAL => {
            *lhs_type.value.as_ref().local < *rhs_type.value.as_ref().local
        }
    }
}

/// Checks if the event corresponding to [`iox2_waitset_guard_h_ref`] was originating from the
/// provided [`iox2_waitset_attachment_id_h_ref`].
///
/// # Safety
///  * `handle` must be valid and non-null.
///  * `guard` must be valid and non-null.
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_attachment_id_has_event_from(
    handle: iox2_waitset_attachment_id_h_ref,
    guard: iox2_waitset_guard_h_ref,
) -> bool {
    handle.assert_non_null();
    guard.assert_non_null();

    let attachment_id = &mut *handle.as_type();
    let guard = &*guard.as_type();

    match attachment_id.service_type {
        iox2_service_type_e::IPC => attachment_id
            .value
            .as_ref()
            .ipc
            .has_event_from(&*guard.value.as_ref().ipc),
        iox2_service_type_e::LOCAL => attachment_id
            .value
            .as_ref()
            .local
            .has_event_from(&*guard.value.as_ref().local),
    }
}

/// Checks if the deadline corresponding to [`iox2_waitset_guard_h_ref`] was originating from the
/// provided [`iox2_waitset_attachment_id_h_ref`].
///
/// # Safety
///  * `handle` must be valid and non-null.
///  * `guard` must be valid and non-null.
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_attachment_id_has_missed_deadline(
    handle: iox2_waitset_attachment_id_h_ref,
    guard: iox2_waitset_guard_h_ref,
) -> bool {
    handle.assert_non_null();
    guard.assert_non_null();

    let attachment_id = &mut *handle.as_type();
    let guard = &*guard.as_type();

    match attachment_id.service_type {
        iox2_service_type_e::IPC => attachment_id
            .value
            .as_ref()
            .ipc
            .has_missed_deadline(&*guard.value.as_ref().ipc),
        iox2_service_type_e::LOCAL => attachment_id
            .value
            .as_ref()
            .local
            .has_missed_deadline(&*guard.value.as_ref().local),
    }
}

/// Creates a new [`iox2_waitset_attachment_id_t`] from an existing [`iox2_waitset_guard_h_ref`].
///
/// # Safety
///  * `guard` must be valid and non-null.
///  * `attachment_id_struct_ptr` must be either null or pointing to a valid uninitialized memory
///    location.
///  * `attachment_id_handle_ptr` must point to a valid uninitialized memory location
///  * `attachment_id_handle_ptr` must be released with [`iox2_waitset_attachment_id_drop()`]
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_attachment_id_from_guard(
    guard: iox2_waitset_guard_h_ref,
    attachment_id_struct_ptr: *mut iox2_waitset_attachment_id_t,
    attachment_id_handle_ptr: *mut iox2_waitset_attachment_id_h,
) {
    guard.assert_non_null();
    debug_assert!(!attachment_id_handle_ptr.is_null());

    let guard = &*guard.as_type();

    let mut attachment_id_struct_ptr = attachment_id_struct_ptr;
    fn no_op(_: *mut iox2_waitset_attachment_id_t) {}
    let mut deleter: fn(*mut iox2_waitset_attachment_id_t) = no_op;
    if attachment_id_struct_ptr.is_null() {
        attachment_id_struct_ptr = iox2_waitset_attachment_id_t::alloc();
        deleter = iox2_waitset_attachment_id_t::dealloc;
    }
    debug_assert!(!attachment_id_struct_ptr.is_null());

    match guard.service_type {
        iox2_service_type_e::IPC => {
            (*attachment_id_struct_ptr).init(
                guard.service_type,
                AttachmentIdUnion::new_ipc(WaitSetAttachmentId::from_guard(
                    &*guard.value.as_ref().ipc,
                )),
                deleter,
            );
        }
        iox2_service_type_e::LOCAL => {
            (*attachment_id_struct_ptr).init(
                guard.service_type,
                AttachmentIdUnion::new_local(WaitSetAttachmentId::from_guard(
                    &*guard.value.as_ref().local,
                )),
                deleter,
            );
        }
    };

    *attachment_id_handle_ptr = (*attachment_id_struct_ptr).as_handle();
}

/// Stores the debug output in the provided `debug_output` variable that must provide enough
/// memory to store the content. The content length can be acquired with
/// [`iox2_waitset_attachment_id_debug_len()`]
///
/// # Safety
///  * `handle` must be valid and non-null.
///  * `debug_output` must be valid and provide enough memory
///  * `debug_len` the provided memory length of `debug_output`
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_attachment_id_debug(
    handle: iox2_waitset_attachment_id_h_ref,
    debug_output: *mut c_char,
    debug_len: c_size_t,
) -> bool {
    handle.assert_non_null();
    debug_assert!(!debug_output.is_null());

    let attachment_id = &mut *handle.as_type();

    let raw_str = match attachment_id.service_type {
        iox2_service_type_e::IPC => format!("{:?}\0", *attachment_id.value.as_mut().ipc),
        iox2_service_type_e::LOCAL => format!("{:?}\0", *attachment_id.value.as_mut().local),
    };

    if debug_len < raw_str.len() {
        return false;
    }

    core::ptr::copy_nonoverlapping(
        raw_str.as_bytes().as_ptr().cast(),
        debug_output,
        raw_str.len(),
    );

    true
}

/// Returns the length of the debug output. Shall be used before calling
/// [`iox2_waitset_attachment_id_debug()`] to acquire enough memory to store the output.
///
/// # Safety
///  * `handle` must be valid and non-null.
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_attachment_id_debug_len(
    handle: iox2_waitset_attachment_id_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let attachment_id = &mut *handle.as_type();

    match attachment_id.service_type {
        iox2_service_type_e::IPC => format!("{:?}\0", *attachment_id.value.as_mut().ipc).len(),
        iox2_service_type_e::LOCAL => format!("{:?}\0", *attachment_id.value.as_mut().local).len(),
    }
}

// END C API
