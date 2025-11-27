// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use crate::{
    api::{c_size_t, iox2_service_type_e, AssertNonNullHandle, HandleToType},
    iox2_event_id_t,
};
use core::ffi::c_void;
use core::mem::ManuallyDrop;
use iceoryx2::port::reader::__InternalEntryHandle;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

// BEGIN types definition

pub(super) union EntryHandleUnion {
    ipc: ManuallyDrop<__InternalEntryHandle<crate::IpcService>>,
    local: ManuallyDrop<__InternalEntryHandle<crate::LocalService>>,
}

impl EntryHandleUnion {
    pub(super) fn new_ipc(entry_handle: __InternalEntryHandle<crate::IpcService>) -> Self {
        Self {
            ipc: ManuallyDrop::new(entry_handle),
        }
    }
    pub(super) fn new_local(entry_handle: __InternalEntryHandle<crate::LocalService>) -> Self {
        Self {
            local: ManuallyDrop::new(entry_handle),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<EntryHandleUnion>
pub struct iox2_entry_handle_storage_t {
    internal: [u8; 40], // magic number obtained with size_of::<Option<EntryHandleUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(EntryHandleUnion)]
pub struct iox2_entry_handle_t {
    service_type: iox2_service_type_e,
    value: iox2_entry_handle_storage_t,
    deleter: fn(*mut iox2_entry_handle_t),
}

impl iox2_entry_handle_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: EntryHandleUnion,
        deleter: fn(*mut iox2_entry_handle_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_entry_handle_h_t;
/// The owning handle for `iox2_entry_handle_t`. Passing the handle to an function transfers the ownership.
pub type iox2_entry_handle_h = *mut iox2_entry_handle_h_t;
/// The non-owning handle for `iox2_entry_handle_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_entry_handle_h_ref = *const iox2_entry_handle_h;

impl AssertNonNullHandle for iox2_entry_handle_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_entry_handle_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_entry_handle_h {
    type Target = *mut iox2_entry_handle_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_entry_handle_h_ref {
    type Target = *mut iox2_entry_handle_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}
// END type definition

// BEGIN C API

/// Copies the value to `value_ptr`. If a `generation_counter_ptr` is passed, a copy of the
/// value's generation counter is stored in it which can be used to check for value updates.
///
/// # Safety
///
/// * `entry_handle_handle` obtained by [`iox2_reader_entry()`](crate::iox2_reader_entry())
/// * `value_ptr` a valid, non-null [`*mut c_void`] pointer
/// * `value_size` the size of the value type
/// * `value_alignment` the alignment of the value type
#[no_mangle]
pub unsafe extern "C" fn iox2_entry_handle_get(
    entry_handle_handle: iox2_entry_handle_h_ref,
    value_ptr: *mut c_void,
    value_size: c_size_t,
    value_alignment: c_size_t,
    generation_counter_ptr: *mut c_void,
) {
    entry_handle_handle.assert_non_null();
    debug_assert!(!value_ptr.is_null());

    let entry_handle = &*entry_handle_handle.as_type();

    match entry_handle.service_type {
        iox2_service_type_e::IPC => entry_handle.value.as_ref().ipc.get(
            value_ptr as *mut u8,
            value_size,
            value_alignment,
            generation_counter_ptr as *mut u64,
        ),
        iox2_service_type_e::LOCAL => entry_handle.value.as_ref().local.get(
            value_ptr as *mut u8,
            value_size,
            value_alignment,
            generation_counter_ptr as *mut u64,
        ),
    };
}

/// Checks if the blackboard value that corresponds to the `generation_counter` is
/// up-to-date.
///
/// # Safety
///
/// * `entry_handle_handle` obtained by [`iox2_reader_entry()`](crate::iox2_reader_entry())
#[no_mangle]
pub unsafe extern "C" fn iox2_entry_handle_is_current(
    entry_handle_handle: iox2_entry_handle_h_ref,
    generation_counter: u64,
) -> bool {
    entry_handle_handle.assert_non_null();

    let entry_handle = &*entry_handle_handle.as_type();

    match entry_handle.service_type {
        iox2_service_type_e::IPC => entry_handle
            .value
            .as_ref()
            .ipc
            .is_current(generation_counter),
        iox2_service_type_e::LOCAL => entry_handle
            .value
            .as_ref()
            .local
            .is_current(generation_counter),
    }
}

/// Returns an id corresponding to the entry which can be used in an event based communication setup.
///
/// # Safety
///
/// * `entry_handle_handle` obtained by [`iox2_reader_entry()`](crate::iox2_reader_entry())
/// * `entry_id` a valid, non-null pointer pointing to a [`iox2_event_id_t`]
#[no_mangle]
pub unsafe extern "C" fn iox2_entry_handle_entry_id(
    entry_handle_handle: iox2_entry_handle_h_ref,
    entry_id: *mut iox2_event_id_t,
) {
    entry_handle_handle.assert_non_null();
    debug_assert!(!entry_id.is_null());

    let entry_handle = &mut *entry_handle_handle.as_type();

    let result = match entry_handle.service_type {
        iox2_service_type_e::IPC => entry_handle.value.as_ref().ipc.entry_id(),
        iox2_service_type_e::LOCAL => entry_handle.value.as_ref().local.entry_id(),
    };

    *entry_id = result.into();
}

/// This function needs to be called to destroy the entry handle!
///
/// # Arguments
///
/// * `entry_handle_handle` - A valid [`iox2_entry_handle_h`]
///
/// # Safety
///
/// * The `entry_handle_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_entry_handle_t`] can be re-used with a call to
///   [`iox2_reader_entry`](crate::iox2_reader_entry)!
#[no_mangle]
pub unsafe extern "C" fn iox2_entry_handle_drop(entry_handle_handle: iox2_entry_handle_h) {
    entry_handle_handle.assert_non_null();

    let entry_handle = &mut *entry_handle_handle.as_type();

    match entry_handle.service_type {
        iox2_service_type_e::IPC => {
            ManuallyDrop::drop(&mut entry_handle.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            ManuallyDrop::drop(&mut entry_handle.value.as_mut().local);
        }
    }
    (entry_handle.deleter)(entry_handle);
}
// END C API
