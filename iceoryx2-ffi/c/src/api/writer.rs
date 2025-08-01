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
    iox2_service_type_e, iox2_unique_writer_id_h, iox2_unique_writer_id_t, AssertNonNullHandle,
    HandleToType, KeyFfi,
};
use core::mem::ManuallyDrop;
use iceoryx2::port::writer::Writer;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

// BEGIN types definition

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

// TODO: entry method + WriterHandle
// END C API
