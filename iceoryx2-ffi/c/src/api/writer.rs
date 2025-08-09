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
    c_size_t, iox2_entry_handle_mut_h, iox2_entry_handle_mut_t, iox2_service_type_e,
    iox2_type_variant_e, iox2_unique_writer_id_h, iox2_unique_writer_id_t, AssertNonNullHandle,
    EntryHandleMutUnion, HandleToType, IntoCInt, KeyFfi, IOX2_OK,
};
use crate::create_type_details;
use core::ffi::{c_char, c_int};
use core::mem::ManuallyDrop;
use iceoryx2::port::writer::{Writer, WriterHandleError};
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_bb_elementary_traits::AsCStr;
use iceoryx2_ffi_macros::{iceoryx2_ffi, CStrRepr};

// BEGIN types definition

#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_entry_handle_mut_error_e {
    ENTRY_DOES_NOT_EXIST = IOX2_OK as isize + 1,
    HANDLE_ALREADY_EXISTS,
}

impl IntoCInt for WriterHandleError {
    fn into_c_int(self) -> c_int {
        (match self {
            WriterHandleError::EntryDoesNotExist => {
                iox2_entry_handle_mut_error_e::ENTRY_DOES_NOT_EXIST
            }
            WriterHandleError::HandleAlreadyExists => {
                iox2_entry_handle_mut_error_e::HANDLE_ALREADY_EXISTS
            }
        }) as c_int
    }
}

pub(super) union WriterUnion {
    ipc: ManuallyDrop<Writer<crate::IpcService, KeyFfi>>,
    local: ManuallyDrop<Writer<crate::LocalService, KeyFfi>>,
}

impl WriterUnion {
    pub(super) fn new_ipc(writer: Writer<crate::IpcService, KeyFfi>) -> Self {
        Self {
            ipc: ManuallyDrop::new(writer),
        }
    }
    pub(super) fn new_local(writer: Writer<crate::LocalService, KeyFfi>) -> Self {
        Self {
            local: ManuallyDrop::new(writer),
        }
    }
}

#[repr(C)]
#[repr(align(16))] // alignment of Option<WriterUnion>
pub struct iox2_writer_storage_t {
    // TODO: adapt size and alignment
    internal: [u8; 1232], // magic number obtained with size_of::<Option<WriterUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(WriterUnion)]
pub struct iox2_writer_t {
    service_type: iox2_service_type_e,
    value: iox2_writer_storage_t,
    deleter: fn(*mut iox2_writer_t),
}

impl iox2_writer_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: WriterUnion,
        deleter: fn(*mut iox2_writer_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_writer_h_t;
/// The owning handle for `iox2_writer_t`. Passing the handle to an function transfers the ownership.
pub type iox2_writer_h = *mut iox2_writer_h_t;
/// The non-owning handle for `iox2_writer_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_writer_h_ref = *const iox2_writer_h;

impl AssertNonNullHandle for iox2_writer_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_writer_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_writer_h {
    type Target = *mut iox2_writer_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_writer_h_ref {
    type Target = *mut iox2_writer_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition

// BEGIN C API

/// Returns a string literal describing the provided [`iox2_entry_handle_mut_error_e`].
///
/// # Arguments
///
/// * `error` - The error value for which a description should be returned
///
/// # Returns
///
/// A pointer to a null-terminated string containing the error message.
/// The string is stored in the .rodata section of the binary.
///
/// # Safety
///
/// The returned pointer must not be modified or freed and is valid as long as the program runs.
#[no_mangle]
pub unsafe extern "C" fn iox2_entry_handle_mut_error_string(
    error: iox2_entry_handle_mut_error_e,
) -> *const c_char {
    error.as_const_cstr().as_ptr() as *const c_char
}

/// Returns the unique port id of the writer.
///
/// # Arguments
///
/// * `writer_handle` obtained by [`iox2_port_factory_writer_builder_create`](crate::iox2_port_factory_writer_builder_create)
/// * `id_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_unique_writer_id_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
/// * `id_handle_ptr` valid pointer to a [`iox2_unique_writer_id_h`].
///
/// # Safety
///
/// * `writer_handle` is valid, non-null and was obtained via [`iox2_port_factory_writer_builder_create`](crate::iox2_port_factory_writer_builder_create)
/// * `id` is valid and non-null
#[no_mangle]
pub unsafe extern "C" fn iox2_writer_id(
    writer_handle: iox2_writer_h_ref,
    id_struct_ptr: *mut iox2_unique_writer_id_t,
    id_handle_ptr: *mut iox2_unique_writer_id_h,
) {
    writer_handle.assert_non_null();
    debug_assert!(!id_handle_ptr.is_null());

    fn no_op(_: *mut iox2_unique_writer_id_t) {}
    let mut deleter: fn(*mut iox2_unique_writer_id_t) = no_op;
    let mut storage_ptr = id_struct_ptr;
    if id_struct_ptr.is_null() {
        deleter = iox2_unique_writer_id_t::dealloc;
        storage_ptr = iox2_unique_writer_id_t::alloc();
    }
    debug_assert!(!storage_ptr.is_null());

    let writer = &mut *writer_handle.as_type();

    let id = match writer.service_type {
        iox2_service_type_e::IPC => writer.value.as_mut().ipc.id(),
        iox2_service_type_e::LOCAL => writer.value.as_mut().local.id(),
    };

    (*storage_ptr).init(id, deleter);
    *id_handle_ptr = (*storage_ptr).as_handle();
}

//// TODO: documentation
#[no_mangle]
pub unsafe extern "C" fn iox2_writer_entry(
    writer_handle: iox2_writer_h_ref,
    entry_handle_mut_struct_ptr: *mut iox2_entry_handle_mut_t,
    entry_handle_mut_handle_ptr: *mut iox2_entry_handle_mut_h,
    key: KeyFfi,
    value_type_name_str: *const c_char,
    value_type_name_len: c_size_t,
    value_size: c_size_t,
    value_alignment: c_size_t,
) -> c_int {
    writer_handle.assert_non_null();
    debug_assert!(!entry_handle_mut_handle_ptr.is_null());

    let init_entry_handle_mut_struct_ptr = |entry_struct_ptr: *mut iox2_entry_handle_mut_t| {
        let mut entry_handle_mut_struct_ptr = entry_struct_ptr;
        fn no_op(_: *mut iox2_entry_handle_mut_t) {}
        let mut deleter: fn(*mut iox2_entry_handle_mut_t) = no_op;
        if entry_handle_mut_struct_ptr.is_null() {
            entry_handle_mut_struct_ptr = iox2_entry_handle_mut_t::alloc();
            deleter = iox2_entry_handle_mut_t::dealloc;
        }
        debug_assert!(!entry_handle_mut_struct_ptr.is_null());

        (entry_handle_mut_struct_ptr, deleter)
    };

    let value_type_details = match create_type_details(
        iox2_type_variant_e::FIXED_SIZE,
        value_type_name_str,
        value_type_name_len,
        value_size,
        value_alignment,
    ) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let writer = &mut *writer_handle.as_type();

    match writer.service_type {
        iox2_service_type_e::IPC => match writer
            .value
            .as_ref()
            .ipc
            .__internal_entry(&key, &value_type_details)
        {
            Ok(handle) => {
                let (entry_handle_mut_struct_ptr, deleter) =
                    init_entry_handle_mut_struct_ptr(entry_handle_mut_struct_ptr);
                (*entry_handle_mut_struct_ptr).init(
                    writer.service_type,
                    EntryHandleMutUnion::new_ipc(handle),
                    deleter,
                );
                *entry_handle_mut_handle_ptr = (*entry_handle_mut_struct_ptr).as_handle();
            }
            Err(error) => return error.into_c_int(),
        },
        iox2_service_type_e::LOCAL => match writer
            .value
            .as_ref()
            .ipc
            .__internal_entry(&key, &value_type_details)
        {
            Ok(handle) => {
                let (entry_handle_mut_struct_ptr, deleter) =
                    init_entry_handle_mut_struct_ptr(entry_handle_mut_struct_ptr);
                (*entry_handle_mut_struct_ptr).init(
                    writer.service_type,
                    EntryHandleMutUnion::new_local(handle),
                    deleter,
                );
                *entry_handle_mut_handle_ptr = (*entry_handle_mut_struct_ptr).as_handle();
            }
            Err(error) => return error.into_c_int(),
        },
    }

    IOX2_OK
}

/// This function needs to be called to destroy the writer!
///
/// # Arguments
///
/// * `writer_handle` - A valid [`iox2_writer_h`]
///
/// # Safety
///
/// * The `writer_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_writer_t`] can be re-used with a call to
///   [`iox2_port_factory_writer_builder_create`](crate::iox2_port_factory_writer_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_writer_drop(writer_handle: iox2_writer_h) {
    writer_handle.assert_non_null();

    let writer = &mut *writer_handle.as_type();

    match writer.service_type {
        iox2_service_type_e::IPC => {
            ManuallyDrop::drop(&mut writer.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            ManuallyDrop::drop(&mut writer.value.as_mut().local);
        }
    }
    (writer.deleter)(writer);
}
// END C API
