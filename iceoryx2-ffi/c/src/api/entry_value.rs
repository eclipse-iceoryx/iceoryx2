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
    iox2_entry_handle_mut_h, iox2_entry_handle_mut_t, iox2_service_type_e, AssertNonNullHandle,
    EntryHandleMutUnion, HandleToType,
};
use core::ffi::c_void;
use core::mem::ManuallyDrop;
use iceoryx2::port::writer::__InternalEntryValueUninit;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

// BEGIN types definition

pub(super) union EntryValueUninitUnion {
    ipc: ManuallyDrop<__InternalEntryValueUninit<crate::IpcService>>,
    local: ManuallyDrop<__InternalEntryValueUninit<crate::LocalService>>,
}

impl EntryValueUninitUnion {
    pub(super) fn new_ipc(entry_value: __InternalEntryValueUninit<crate::IpcService>) -> Self {
        Self {
            ipc: ManuallyDrop::new(entry_value),
        }
    }
    pub(super) fn new_local(entry_value: __InternalEntryValueUninit<crate::LocalService>) -> Self {
        Self {
            local: ManuallyDrop::new(entry_value),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<EntryValueUninitUnion>
pub struct iox2_entry_value_storage_t {
    internal: [u8; 48], // magic number obtained with size_of::<Option<EntryValueUninitUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(EntryValueUninitUnion)]
pub struct iox2_entry_value_t {
    service_type: iox2_service_type_e,
    value: iox2_entry_value_storage_t,
    deleter: fn(*mut iox2_entry_value_t),
}

impl iox2_entry_value_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: EntryValueUninitUnion,
        deleter: fn(*mut iox2_entry_value_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_entry_value_h_t;
/// The owning handle for `iox2_entry_value_t`. Passing the handle to a function transfers the ownership.
pub type iox2_entry_value_h = *mut iox2_entry_value_h_t;
/// The non-owning handle for `iox2_entry_value_t`. Passing the handle to a function does not transfer the ownership.
pub type iox2_entry_value_h_ref = *const iox2_entry_value_h;

impl AssertNonNullHandle for iox2_entry_value_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_entry_value_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_entry_value_h {
    type Target = *mut iox2_entry_value_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_entry_value_h_ref {
    type Target = *mut iox2_entry_value_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}
// END type definition

// BEGIN C API

/// cbindgen:ignore
/// Internal API - do not use
/// # Safety
///
/// * `source_struct_ptr` must not be `null` and the struct it is pointing to must be initialized and valid, i.e. not moved or dropped.
/// * `dest_struct_ptr` must not be `null` and the struct it is pointing to must not contain valid data, i.e. initialized. It can be moved or dropped, though.
/// * `dest_handle_ptr` must not be `null`
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn iox2_entry_value_move(
    source_struct_ptr: *mut iox2_entry_value_t,
    dest_struct_ptr: *mut iox2_entry_value_t,
    dest_handle_ptr: *mut iox2_entry_value_h,
) {
    debug_assert!(!source_struct_ptr.is_null());
    debug_assert!(!dest_struct_ptr.is_null());
    debug_assert!(!dest_handle_ptr.is_null());

    let source = &mut *source_struct_ptr;
    let dest = &mut *dest_struct_ptr;

    dest.service_type = source.service_type;
    dest.value.init(
        source
            .value
            .as_option_mut()
            .take()
            .expect("Source must have a valid sample"),
    );
    dest.deleter = source.deleter;

    *dest_handle_ptr = (*dest_struct_ptr).as_handle();
}

/// Acquires the entrie's mutable value.
///
/// # Safety
///
/// * `entry_value_handle` obtained by [`iox2_entry_handle_mut_loan_uninit()`](crate::iox2_entry_handle_mut_loan_uninit())
/// * `value_ptr` a valid, non-null pointer pointing to a [`*mut c_void`] pointer.
#[no_mangle]
pub unsafe extern "C" fn iox2_entry_value_mut(
    entry_value_handle: iox2_entry_value_h_ref,
    value_ptr: *mut *mut c_void,
) {
    entry_value_handle.assert_non_null();
    debug_assert!(!value_ptr.is_null());

    let entry_value = &mut *entry_value_handle.as_type();

    *value_ptr = match entry_value.service_type {
        iox2_service_type_e::IPC => entry_value.value.as_ref().ipc.write_cell().cast(),
        iox2_service_type_e::LOCAL => entry_value.value.as_ref().local.write_cell().cast(),
    };
}

/// Consumes the entry value, makes the new value readable for [`iox2_reader_t`](crate::iox2_reader_t) and returns the original entry handle mut.
///
/// # Safety
///
/// * `entry_value_handle` obtained by [`iox2_entry_handle_mut_loan_uninit()`](crate::iox2_entry_handle_mut_loan_uninit()), it's invalid after the return of this function
/// * `entry_handle_mut_struct_ptr` must be either a NULL pointer or a pointer to a valid [`iox2_entry_handle_mut_t`](crate::iox2_entry_handle_mut_t)
/// * `entry_handle_mut_handle_ptr` a valid, non-null [`*mut iox2_entry_handle_mut_h`] pointer which will be initialized by this function call
#[no_mangle]
pub unsafe extern "C" fn iox2_entry_value_update(
    entry_value_handle: iox2_entry_value_h,
    entry_handle_mut_struct_ptr: *mut iox2_entry_handle_mut_t,
    entry_handle_mut_handle_ptr: *mut iox2_entry_handle_mut_h,
) {
    entry_value_handle.assert_non_null();
    debug_assert!(!entry_handle_mut_handle_ptr.is_null());

    let init_entry_handle_mut_struct_ptr =
        |entry_handle_mut_struct_ptr: *mut iox2_entry_handle_mut_t| {
            let mut entry_handle_mut_struct_ptr = entry_handle_mut_struct_ptr;
            fn no_op(_: *mut iox2_entry_handle_mut_t) {}
            let mut deleter: fn(*mut iox2_entry_handle_mut_t) = no_op;
            if entry_handle_mut_struct_ptr.is_null() {
                entry_handle_mut_struct_ptr = iox2_entry_handle_mut_t::alloc();
                deleter = iox2_entry_handle_mut_t::dealloc;
            }
            debug_assert!(!entry_handle_mut_struct_ptr.is_null());

            (entry_handle_mut_struct_ptr, deleter)
        };

    let entry_value_struct = &mut *entry_value_handle.as_type();
    let service_type = entry_value_struct.service_type;
    let entry_value = entry_value_struct
        .value
        .as_option_mut()
        .take()
        .unwrap_or_else(|| {
            panic!("Trying to use an invalid 'iox2_entry_value_h'!");
        });
    (entry_value_struct.deleter)(entry_value_struct);

    match service_type {
        iox2_service_type_e::IPC => {
            let entry_value = ManuallyDrop::into_inner(entry_value.ipc);
            let entry_handle_mut = entry_value.update();
            let (entry_handle_mut_struct_ptr, deleter) =
                init_entry_handle_mut_struct_ptr(entry_handle_mut_struct_ptr);
            (*entry_handle_mut_struct_ptr).init(
                service_type,
                EntryHandleMutUnion::new_ipc(entry_handle_mut),
                deleter,
            );
            *entry_handle_mut_handle_ptr = (*entry_handle_mut_struct_ptr).as_handle();
        }
        iox2_service_type_e::LOCAL => {
            let entry_value = ManuallyDrop::into_inner(entry_value.local);
            let entry_handle_mut = entry_value.update();
            let (entry_handle_mut_struct_ptr, deleter) =
                init_entry_handle_mut_struct_ptr(entry_handle_mut_struct_ptr);
            (*entry_handle_mut_struct_ptr).init(
                service_type,
                EntryHandleMutUnion::new_local(entry_handle_mut),
                deleter,
            );
            *entry_handle_mut_handle_ptr = (*entry_handle_mut_struct_ptr).as_handle();
        }
    }
}

/// Consumes and discards the entry value and returns the original entry handle mut.
///
/// # Safety
///
/// * `entry_value_handle` obtained by [`iox2_entry_handle_mut_loan_uninit()`](crate::iox2_entry_handle_mut_loan_uninit()), it's invalid after the return of this function
/// * `entry_handle_mut_struct_ptr` must be either a NULL pointer or a pointer to a valid [`iox2_entry_handle_mut_t`]
/// * `entry_handle_mut_handle_ptr` a valid, non-null [`*mut iox2_entry_handle_mut_h`] pointer which will be initialized by this function call
#[no_mangle]
pub unsafe extern "C" fn iox2_entry_value_discard(
    entry_value_handle: iox2_entry_value_h,
    entry_handle_mut_struct_ptr: *mut iox2_entry_handle_mut_t,
    entry_handle_mut_handle_ptr: *mut iox2_entry_handle_mut_h,
) {
    entry_value_handle.assert_non_null();
    debug_assert!(!entry_handle_mut_handle_ptr.is_null());

    let init_entry_handle_mut_struct_ptr =
        |entry_handle_mut_struct_ptr: *mut iox2_entry_handle_mut_t| {
            let mut entry_handle_mut_struct_ptr = entry_handle_mut_struct_ptr;
            fn no_op(_: *mut iox2_entry_handle_mut_t) {}
            let mut deleter: fn(*mut iox2_entry_handle_mut_t) = no_op;
            if entry_handle_mut_struct_ptr.is_null() {
                entry_handle_mut_struct_ptr = iox2_entry_handle_mut_t::alloc();
                deleter = iox2_entry_handle_mut_t::dealloc;
            }
            debug_assert!(!entry_handle_mut_struct_ptr.is_null());

            (entry_handle_mut_struct_ptr, deleter)
        };

    let entry_value_struct = &mut *entry_value_handle.as_type();
    let service_type = entry_value_struct.service_type;
    let entry_value = entry_value_struct
        .value
        .as_option_mut()
        .take()
        .unwrap_or_else(|| {
            panic!("Trying to use an invalid 'iox2_entry_value_h'!");
        });
    (entry_value_struct.deleter)(entry_value_struct);

    match service_type {
        iox2_service_type_e::IPC => {
            let entry_value = ManuallyDrop::into_inner(entry_value.ipc);
            let entry_handle_mut = entry_value.discard();
            let (entry_handle_mut_struct_ptr, deleter) =
                init_entry_handle_mut_struct_ptr(entry_handle_mut_struct_ptr);
            (*entry_handle_mut_struct_ptr).init(
                service_type,
                EntryHandleMutUnion::new_ipc(entry_handle_mut),
                deleter,
            );
            *entry_handle_mut_handle_ptr = (*entry_handle_mut_struct_ptr).as_handle();
        }
        iox2_service_type_e::LOCAL => {
            let entry_value = ManuallyDrop::into_inner(entry_value.local);
            let entry_handle_mut = entry_value.discard();
            let (entry_handle_mut_struct_ptr, deleter) =
                init_entry_handle_mut_struct_ptr(entry_handle_mut_struct_ptr);
            (*entry_handle_mut_struct_ptr).init(
                service_type,
                EntryHandleMutUnion::new_local(entry_handle_mut),
                deleter,
            );
            *entry_handle_mut_handle_ptr = (*entry_handle_mut_struct_ptr).as_handle();
        }
    }
}

/// This function needs to be called to destroy the entry value!
///
/// # Arguments
///
/// * `entry_value_handle` - A valid [`iox2_entry_value_h`]
///
/// # Safety
///
/// * The `entry_value_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_entry_value_t`] can be re-used with a call to
///   [`iox2_entry_handle_mut_loan_uninit()`](crate::iox2_entry_handle_mut_loan_uninit())!
#[no_mangle]
pub unsafe extern "C" fn iox2_entry_value_drop(entry_value_handle: iox2_entry_value_h) {
    entry_value_handle.assert_non_null();

    let entry_value = &mut *entry_value_handle.as_type();

    match entry_value.service_type {
        iox2_service_type_e::IPC => {
            if let Some(mut value) = entry_value.take() {
                ManuallyDrop::drop(&mut value.ipc);
                (entry_value.deleter)(entry_value);
            }
        }
        iox2_service_type_e::LOCAL => {
            if let Some(mut value) = entry_value.take() {
                ManuallyDrop::drop(&mut value.local);
                (entry_value.deleter)(entry_value);
            }
        }
    }
}
// END C API
