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
    iox2_service_type_e, iox2_writer_h, iox2_writer_t, AssertNonNullHandle, HandleToType, IntoCInt,
    KeyFFi, WriterUnion, IOX2_OK,
};

use iceoryx2::port::writer::WriterCreateError;
use iceoryx2::service::port_factory::writer::PortFactoryWriter;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_bb_elementary_traits::AsCStr;
use iceoryx2_ffi_macros::iceoryx2_ffi;
use iceoryx2_ffi_macros::CStrRepr;

use core::ffi::{c_char, c_int};
use core::mem::ManuallyDrop;

// BEGIN types definition

#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_writer_create_error_e {
    EXCEEDS_MAX_SUPPORTED_WRITERS = IOX2_OK as isize + 1,
    INTERNAL_FAILURE,
}

impl IntoCInt for WriterCreateError {
    fn into_c_int(self) -> c_int {
        (match self {
            WriterCreateError::ExceedsMaxSupportedWriters => {
                iox2_writer_create_error_e::EXCEEDS_MAX_SUPPORTED_WRITERS
            }
            WriterCreateError::InternalFailure => iox2_writer_create_error_e::INTERNAL_FAILURE,
        }) as c_int
    }
}

pub(super) union PortFactoryWriterBuilderUnion {
    ipc: ManuallyDrop<PortFactoryWriter<'static, crate::IpcService, KeyFfi>>,
    local: ManuallyDrop<PortFactoryWriter<'static, crate::LocalService, KeyFfi>>,
}

impl PortFactoryWriterBuilderUnion {
    pub(super) fn new_ipc(
        port_factory: PortFactoryWriter<'static, crate::IpcService, KeyFfi>,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(port_factory),
        }
    }
    pub(super) fn new_local(
        port_factory: PortFactoryWriter<'static, crate::LocalService, KeyFfi>,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(port_factory),
        }
    }
}

#[repr(C)]
#[repr(align(16))] // alignment of Option<PortFactoryWriterBuilderUnion>
pub struct iox2_port_factory_writer_builder_storage_t {
    // TODO: adapt size and alignment
    internal: [u8; 112], // magic number obtained with size_of::<Option<PortFactoryWriterBuilderUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(PortFactoryWriterBuilderUnion)]
pub struct iox2_port_factory_writer_builder_t {
    service_type: iox2_service_type_e,
    value: iox2_port_factory_writer_builder_storage_t,
    deleter: fn(*mut iox2_port_factory_writer_builder_t),
}

impl iox2_port_factory_writer_builder_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: PortFactoryWriterBuilderUnion,
        deleter: fn(*mut iox2_port_factory_writer_builder_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_port_factory_writer_builder_h_t;
/// The owning handle for `iox2_port_factory_writer_builder_t`. Passing the handle to an function transfers the ownership.
pub type iox2_port_factory_writer_builder_h = *mut iox2_port_factory_writer_builder_h_t;
/// The non-owning handle for `iox2_port_factory_writer_builder_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_port_factory_writer_builder_h_ref = *const iox2_port_factory_writer_builder_h;

impl AssertNonNullHandle for iox2_port_factory_writer_builder_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_port_factory_writer_builder_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_port_factory_writer_builder_h {
    type Target = *mut iox2_port_factory_writer_builder_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_port_factory_writer_builder_h_ref {
    type Target = *mut iox2_port_factory_writer_builder_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition

// BEGIN C API

/// Returns a string literal describing the provided [`iox2_writer_create_error_e`].
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
pub unsafe extern "C" fn iox2_writer_create_error_string(
    error: iox2_writer_create_error_e,
) -> *const c_char {
    error.as_const_cstr().as_ptr() as *const c_char
}

/// Creates a writer and consumes the builder
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_writer_builder_h`] obtained by [`iox2_port_factory_blackboard_writer_builder`](crate::iox2_port_factory_blackboard_writer_builder).
/// * `writer_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_writer_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `writer_handle_ptr` - An uninitialized or dangling [`iox2_writer_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_writer_create_error_e`] otherwise.
///
/// # Safety
///
/// * The `port_factory_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_port_factory_writer_builder_t`]
///   can be re-used with a call to  [`iox2_port_factory_blackboard_writer_builder`](crate::iox2_port_factory_blackboard_writer_builder)!
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_writer_builder_create(
    port_factory_handle: iox2_port_factory_writer_builder_h,
    writer_struct_ptr: *mut iox2_writer_t,
    writer_handle_ptr: *mut iox2_writer_h,
) -> c_int {
    debug_assert!(!port_factory_handle.is_null());
    debug_assert!(!writer_handle_ptr.is_null());

    let mut writer_struct_ptr = writer_struct_ptr;
    fn no_op(_: *mut iox2_writer_t) {}
    let mut deleter: fn(*mut iox2_writer_t) = no_op;
    if writer_struct_ptr.is_null() {
        writer_struct_ptr = iox2_writer_t::alloc();
        deleter = iox2_writer_t::dealloc;
    }
    debug_assert!(!writer_struct_ptr.is_null());

    let writer_builder_struct = unsafe { &mut *port_factory_handle.as_type() };
    let service_type = writer_builder_struct.service_type;
    let writer_builder = writer_builder_struct
        .value
        .as_option_mut()
        .take()
        .unwrap_or_else(|| {
            panic!("Trying to use an invalid 'iox2_port_factory_writer_builder_h'!")
        });
    (writer_builder_struct.deleter)(writer_builder_struct);

    match service_type {
        iox2_service_type_e::IPC => {
            let writer_builder = ManuallyDrop::into_inner(writer_builder.ipc);

            match writer_builder.create() {
                Ok(writer) => {
                    (*writer_struct_ptr).init(service_type, WriterUnion::new_ipc(writer), deleter);
                }
                Err(error) => {
                    return error.into_c_int();
                }
            }
        }
        iox2_service_type_e::LOCAL => {
            let writer_builder = ManuallyDrop::into_inner(writer_builder.local);

            match writer_builder.create() {
                Ok(writer) => {
                    (*writer_struct_ptr).init(
                        service_type,
                        WriterUnion::new_local(writer),
                        deleter,
                    );
                }
                Err(error) => {
                    return error.into_c_int();
                }
            }
        }
    }

    *writer_handle_ptr = (*writer_struct_ptr).as_handle();

    IOX2_OK
}

// END C API
