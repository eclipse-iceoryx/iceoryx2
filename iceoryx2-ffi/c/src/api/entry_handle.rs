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

use crate::api::{
    c_size_t, iox2_service_type_e, AssertNonNullHandle, HandleToType, KeyFfi, ValueFfi,
};
use core::ffi::c_void;
use core::mem::ManuallyDrop;
use iceoryx2::port::reader::{ReaderHandle, __InternalReaderHandle};
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

// BEGIN types definition

pub(super) union EntryHandleUnion {
    ipc: ManuallyDrop<__InternalReaderHandle<crate::IpcService>>,
    local: ManuallyDrop<__InternalReaderHandle<crate::LocalService>>,
}

impl EntryHandleUnion {
    pub(super) fn new_ipc(entry_handle: __InternalReaderHandle<crate::IpcService>) -> Self {
        Self {
            ipc: ManuallyDrop::new(entry_handle),
        }
    }
    pub(super) fn new_local(entry_handle: __InternalReaderHandle<crate::LocalService>) -> Self {
        Self {
            local: ManuallyDrop::new(entry_handle),
        }
    }
}

#[repr(C)]
#[repr(align(16))] // alignment of Option<EntryHandleUnion>
pub struct iox2_entry_handle_storage_t {
    // TODO: adapt size and alignment
    internal: [u8; 1232], // magic number obtained with size_of::<Option<EntryHandleUnion>>()
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

// TODO: documentation
#[no_mangle]
pub unsafe extern "C" fn iox2_entry_handle_get(
    entry_handle_handle: iox2_entry_handle_h_ref,
    value_ptr: *mut c_void,
    value_size: c_size_t,
    value_alignment: c_size_t,
) {
    entry_handle_handle.assert_non_null();
    debug_assert!(!value_ptr.is_null());

    let entry_handle = &*entry_handle_handle.as_type();

    match entry_handle.service_type {
        iox2_service_type_e::IPC => {
            entry_handle
                .value
                .as_ref()
                .ipc
                .get(value_ptr as *mut u8, value_size, value_alignment)
        }
        iox2_service_type_e::LOCAL => {
            entry_handle
                .value
                .as_ref()
                .local
                .get(value_ptr as *mut u8, value_size, value_alignment)
        }
    };
}

// TODO: entry_id

// TODO: documentation
#[no_mangle]
pub unsafe extern "C" fn iox2_entry_handle_drop(entry_handle_handle: iox2_entry_handle_h) {
    println!("iox2_entry_handle_drop");
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
