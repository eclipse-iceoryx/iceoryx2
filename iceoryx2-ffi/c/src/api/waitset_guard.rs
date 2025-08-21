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

use core::mem::ManuallyDrop;

use crate::iox2_service_type_e;
use iceoryx2::waitset::WaitSetGuard;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use super::{AssertNonNullHandle, HandleToType};

// BEGIN types definition
pub(crate) union GuardUnion {
    pub(crate) ipc: ManuallyDrop<WaitSetGuard<'static, 'static, crate::IpcService>>,
    pub(crate) local: ManuallyDrop<WaitSetGuard<'static, 'static, crate::LocalService>>,
}

impl GuardUnion {
    pub(super) fn new_ipc(guard: WaitSetGuard<'static, 'static, crate::IpcService>) -> Self {
        Self {
            ipc: ManuallyDrop::new(guard),
        }
    }

    pub(super) fn new_local(guard: WaitSetGuard<'static, 'static, crate::LocalService>) -> Self {
        Self {
            local: ManuallyDrop::new(guard),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<GuardUnion>
pub struct iox2_waitset_guard_storage_t {
    internal: [u8; 56], // magic number obtained with size_of::<Option<GuardUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(GuardUnion)]
pub struct iox2_waitset_guard_t {
    pub(crate) service_type: iox2_service_type_e,
    pub(crate) value: iox2_waitset_guard_storage_t,
    deleter: fn(*mut iox2_waitset_guard_t),
}

impl iox2_waitset_guard_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: GuardUnion,
        deleter: fn(*mut iox2_waitset_guard_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_waitset_guard_h_t;
/// The owning handle for `iox2_waitset_guard_t`. Passing the handle to an function transfers the ownership.
pub type iox2_waitset_guard_h = *mut iox2_waitset_guard_h_t;
/// The non-owning handle for `iox2_attachment_id_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_waitset_guard_h_ref = *const iox2_waitset_guard_h;

impl AssertNonNullHandle for iox2_waitset_guard_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_waitset_guard_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_waitset_guard_h {
    type Target = *mut iox2_waitset_guard_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_waitset_guard_h_ref {
    type Target = *mut iox2_waitset_guard_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}
// END type definition

// BEGIN C API
/// Drops a [`iox2_waitset_guard_h`] that was successfully acquired with
/// * [`iox2_waitset_attach_interval()`](crate::iox2_waitset_attach_interval())
/// * [`iox2_waitset_attach_deadline()`](crate::iox2_waitset_attach_deadline())
/// * [`iox2_waitset_attach_notification()`](crate::iox2_waitset_attach_notification())
///
/// # Safety
///
/// * `handle` must be valid and non null
#[no_mangle]
pub unsafe extern "C" fn iox2_waitset_guard_drop(handle: iox2_waitset_guard_h) {
    handle.assert_non_null();

    let guard = &mut *handle.as_type();

    match guard.service_type {
        iox2_service_type_e::IPC => {
            ManuallyDrop::drop(&mut guard.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            ManuallyDrop::drop(&mut guard.value.as_mut().local);
        }
    }
    (guard.deleter)(guard);
}
// END C API
