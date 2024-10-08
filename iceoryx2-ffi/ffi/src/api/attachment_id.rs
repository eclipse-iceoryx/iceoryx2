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

use std::mem::ManuallyDrop;

use iceoryx2::{
    prelude::AttachmentId,
    service::{ipc, local},
};
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use crate::{iox2_guard_h_ref, iox2_service_type_e};

use super::{AssertNonNullHandle, HandleToType};

// BEGIN types definition
pub(crate) union AttachmentIdUnion {
    ipc: ManuallyDrop<AttachmentId<ipc::Service>>,
    local: ManuallyDrop<AttachmentId<local::Service>>,
}

impl AttachmentIdUnion {
    pub(crate) fn new_ipc(attachment: AttachmentId<ipc::Service>) -> Self {
        Self {
            ipc: ManuallyDrop::new(attachment),
        }
    }

    pub(crate) fn new_local(attachment: AttachmentId<local::Service>) -> Self {
        Self {
            local: ManuallyDrop::new(attachment),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<AttachmentIdUnion>
pub struct iox2_attachment_id_storage_t {
    internal: [u8; 32], // magic number obtained with size_of::<Option<AttachmentIdUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(AttachmentIdUnion)]
pub struct iox2_attachment_id_t {
    service_type: iox2_service_type_e,
    value: iox2_attachment_id_storage_t,
    deleter: fn(*mut iox2_attachment_id_t),
}

impl iox2_attachment_id_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: AttachmentIdUnion,
        deleter: fn(*mut iox2_attachment_id_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_attachment_id_h_t;
/// The owning handle for `iox2_attachment_id_t`. Passing the handle to an function transfers the ownership.
pub type iox2_attachment_id_h = *mut iox2_attachment_id_h_t;
/// The non-owning handle for `iox2_attachment_id_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_attachment_id_h_ref = *const iox2_attachment_id_h;

impl AssertNonNullHandle for iox2_attachment_id_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_attachment_id_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_attachment_id_h {
    type Target = *mut iox2_attachment_id_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_attachment_id_h_ref {
    type Target = *mut iox2_attachment_id_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}
// END type definition

// BEGIN C API
/// Release an [`iox2_attachment_id_h`] that was acquired by calling either
/// * [`iox2_waitset_run()`](crate::iox2_waitset_run())
/// * [`iox2_waitset_run_once()`](crate::iox2_waitset_run_once())
///
/// # Safety
///  * `handle` must be valid and provided by the previously mentioned functions.
#[no_mangle]
pub unsafe extern "C" fn iox2_attachment_id_drop(handle: iox2_attachment_id_h) {
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

/// Checks if two provided [`iox2_attachment_id_h_ref`] are semantically equal.
///
/// # Safety
///  * `lhs` must be valid and non-null.
///  * `rhs` must be valid and non-null.
#[no_mangle]
pub unsafe extern "C" fn iox2_attachment_id_equal(
    lhs: iox2_attachment_id_h_ref,
    rhs: iox2_attachment_id_h_ref,
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

/// Checks the ordering of two provided [`iox2_attachment_id_h_ref`].
///
/// # Safety
///  * `lhs` must be valid and non-null.
///  * `rhs` must be valid and non-null.
#[no_mangle]
pub unsafe extern "C" fn iox2_attachment_id_less(
    lhs: iox2_attachment_id_h_ref,
    rhs: iox2_attachment_id_h_ref,
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

/// Checks if the event corresponding to [`iox2_guard_h_ref`] was originating from the
/// provided [`iox2_attachment_id_h_ref`].
///
/// # Safety
///  * `handle` must be valid and non-null.
///  * `guard` must be valid and non-null.
#[no_mangle]
pub unsafe extern "C" fn iox2_attachment_id_event_from(
    handle: iox2_attachment_id_h_ref,
    guard: iox2_guard_h_ref,
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
            .event_from(&*guard.value.as_ref().ipc),
        iox2_service_type_e::LOCAL => attachment_id
            .value
            .as_ref()
            .local
            .event_from(&*guard.value.as_ref().local),
    }
}

/// Checks if the deadline corresponding to [`iox2_guard_h_ref`] was originating from the
/// provided [`iox2_attachment_id_h_ref`].
///
/// # Safety
///  * `handle` must be valid and non-null.
///  * `guard` must be valid and non-null.
#[no_mangle]
pub unsafe extern "C" fn iox2_attachment_id_deadline_from(
    handle: iox2_attachment_id_h_ref,
    guard: iox2_guard_h_ref,
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
            .deadline_from(&*guard.value.as_ref().ipc),
        iox2_service_type_e::LOCAL => attachment_id
            .value
            .as_ref()
            .local
            .deadline_from(&*guard.value.as_ref().local),
    }
}

/// Creates a new [`iox2_attachment_id_t`] from an existing [`iox2_guard_h_ref`].
///
/// # Safety
///  * `guard` must be valid and non-null.
///  * `attachment_id_struct_ptr` must be either null or pointing to a valid uninitialized memory
///    location.
///  * `attachment_id_handle_ptr` must point to a valid uninitialized memory location
///  * `attachment_id_handle_ptr` must be released with [`iox2_attachment_id_drop()`]
#[no_mangle]
pub unsafe extern "C" fn iox2_attachment_id_from_guard(
    guard: iox2_guard_h_ref,
    attachment_id_struct_ptr: *mut iox2_attachment_id_t,
    attachment_id_handle_ptr: *mut iox2_attachment_id_h,
) {
    guard.assert_non_null();
    debug_assert!(!attachment_id_handle_ptr.is_null());

    let guard = &*guard.as_type();

    let mut attachment_id_struct_ptr = attachment_id_struct_ptr;
    fn no_op(_: *mut iox2_attachment_id_t) {}
    let mut deleter: fn(*mut iox2_attachment_id_t) = no_op;
    if attachment_id_struct_ptr.is_null() {
        attachment_id_struct_ptr = iox2_attachment_id_t::alloc();
        deleter = iox2_attachment_id_t::dealloc;
    }
    debug_assert!(!attachment_id_struct_ptr.is_null());

    match guard.service_type {
        iox2_service_type_e::IPC => {
            (*attachment_id_struct_ptr).init(
                guard.service_type,
                AttachmentIdUnion::new_ipc(AttachmentId::from_guard(&*guard.value.as_ref().ipc)),
                deleter,
            );
        }
        iox2_service_type_e::LOCAL => {
            (*attachment_id_struct_ptr).init(
                guard.service_type,
                AttachmentIdUnion::new_local(AttachmentId::from_guard(
                    &*guard.value.as_ref().local,
                )),
                deleter,
            );
        }
    };

    *attachment_id_handle_ptr = (*attachment_id_struct_ptr).as_handle();
}
// END C API
