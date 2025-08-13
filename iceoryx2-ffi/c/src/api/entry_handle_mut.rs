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
    c_size_t, iox2_entry_value_h, iox2_entry_value_t, iox2_service_type_e, AssertNonNullHandle,
    EntryValueUninitUnion, HandleToType, KeyFfi, ValueFfi,
};
use core::ffi::c_void;
use core::mem::ManuallyDrop;
use iceoryx2::port::writer::{WriterHandle, __InternalWriterHandle};
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

// BEGIN types definition

pub(super) union EntryHandleMutUnion {
    ipc: ManuallyDrop<__InternalWriterHandle<crate::IpcService>>,
    local: ManuallyDrop<__InternalWriterHandle<crate::LocalService>>,
}

impl EntryHandleMutUnion {
    pub(super) fn new_ipc(entry_handle: __InternalWriterHandle<crate::IpcService>) -> Self {
        Self {
            ipc: ManuallyDrop::new(entry_handle),
        }
    }
    pub(super) fn new_local(entry_handle: __InternalWriterHandle<crate::LocalService>) -> Self {
        Self {
            local: ManuallyDrop::new(entry_handle),
        }
    }
}

#[repr(C)]
#[repr(align(16))] // alignment of Option<EntryHandleMutUnion>
pub struct iox2_entry_handle_mut_storage_t {
    // TODO: adapt size and alignment
    internal: [u8; 1232], // magic number obtained with size_of::<Option<EntryHandleMutUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(EntryHandleMutUnion)]
pub struct iox2_entry_handle_mut_t {
    service_type: iox2_service_type_e,
    value: iox2_entry_handle_mut_storage_t,
    deleter: fn(*mut iox2_entry_handle_mut_t),
}

impl iox2_entry_handle_mut_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: EntryHandleMutUnion,
        deleter: fn(*mut iox2_entry_handle_mut_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_entry_handle_mut_h_t;
/// The owning handle for `iox2_entry_handle_mut_t`. Passing the handle to an function transfers the ownership.
pub type iox2_entry_handle_mut_h = *mut iox2_entry_handle_mut_h_t;
/// The non-owning handle for `iox2_entry_handle_mut_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_entry_handle_mut_h_ref = *const iox2_entry_handle_mut_h;

impl AssertNonNullHandle for iox2_entry_handle_mut_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_entry_handle_mut_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_entry_handle_mut_h {
    type Target = *mut iox2_entry_handle_mut_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_entry_handle_mut_h_ref {
    type Target = *mut iox2_entry_handle_mut_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}
// END type definition

// BEGIN C API

#[no_mangle]
pub unsafe extern "C" fn iox2_entry_handle_mut_loan_uninit(
    entry_handle_mut_handle: iox2_entry_handle_mut_h,
    entry_value_struct_ptr: *mut iox2_entry_value_t,
    entry_value_handle_ptr: *mut iox2_entry_value_h,
    value_size: usize,
    value_alignment: usize,
) {
    entry_handle_mut_handle.assert_non_null();
    debug_assert!(!entry_value_handle_ptr.is_null());

    let init_entry_value_struct_ptr = |entry_value_struct_ptr: *mut iox2_entry_value_t| {
        let mut entry_value_struct_ptr = entry_value_struct_ptr;
        fn no_op(_: *mut iox2_entry_value_t) {}
        let mut deleter: fn(*mut iox2_entry_value_t) = no_op;
        if entry_value_struct_ptr.is_null() {
            entry_value_struct_ptr = iox2_entry_value_t::alloc();
            deleter = iox2_entry_value_t::dealloc;
        }
        debug_assert!(!entry_value_struct_ptr.is_null());

        (entry_value_struct_ptr, deleter)
    };

    let entry_handle_mut_struct = &mut *entry_handle_mut_handle.as_type();
    let service_type = entry_handle_mut_struct.service_type;
    let entry_handle_mut = entry_handle_mut_struct
        .value
        .as_option_mut()
        .take()
        .unwrap_or_else(|| {
            panic!("Trying to use an invalid 'iox2_entry_handle_mut_h'!");
        });
    (entry_handle_mut_struct.deleter)(entry_handle_mut_struct);

    match service_type {
        iox2_service_type_e::IPC => {
            let entry_handle_mut = ManuallyDrop::into_inner(entry_handle_mut.ipc);
            let entry_value_uninit = entry_handle_mut.loan_uninit(value_size, value_alignment);
            let (entry_value_struct_ptr, deleter) =
                init_entry_value_struct_ptr(entry_value_struct_ptr);
            (*entry_value_struct_ptr).init(
                service_type,
                EntryValueUninitUnion::new_ipc(entry_value_uninit),
                deleter,
            );
            *entry_value_handle_ptr = (*entry_value_struct_ptr).as_handle();
        }
        iox2_service_type_e::LOCAL => {
            let entry_handle_mut = ManuallyDrop::into_inner(entry_handle_mut.ipc);
            let entry_value_uninit = entry_handle_mut.loan_uninit(value_size, value_alignment);
            let (entry_value_struct_ptr, deleter) =
                init_entry_value_struct_ptr(entry_value_struct_ptr);
            (*entry_value_struct_ptr).init(
                service_type,
                EntryValueUninitUnion::new_local(entry_value_uninit),
                deleter,
            );
            *entry_value_handle_ptr = (*entry_value_struct_ptr).as_handle();
        }
    }
}

// TODO: entry_id

// TODO: loan_uninit consumes entry_handle_mut, so no drop? what happens when loan_uninit was never
// called?
#[no_mangle]
pub unsafe extern "C" fn iox2_entry_handle_mut_drop(
    entry_handle_mut_handle: iox2_entry_handle_mut_h,
) {
    println!("iox2_entry_handle_mut_drop");
    entry_handle_mut_handle.assert_non_null();

    let entry_handle_mut = &mut *entry_handle_mut_handle.as_type();

    match entry_handle_mut.service_type {
        iox2_service_type_e::IPC => {
            if let Some(mut handle) = entry_handle_mut.take() {
                ManuallyDrop::drop(&mut handle.ipc);
                (entry_handle_mut.deleter)(entry_handle_mut);
            }
            //ManuallyDrop::drop(&mut entry_handle_mut.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            //ManuallyDrop::drop(&mut entry_handle_mut.value.as_mut().local);
            if let Some(mut handle) = entry_handle_mut.take() {
                ManuallyDrop::drop(&mut handle.local);
                (entry_handle_mut.deleter)(entry_handle_mut);
            }
        }
    }
    //(entry_handle_mut.deleter)(entry_handle_mut);
}
// END C API
