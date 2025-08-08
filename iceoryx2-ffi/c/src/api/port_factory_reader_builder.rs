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
    iox2_reader_h, iox2_reader_t, iox2_service_type_e, AssertNonNullHandle, HandleToType, IntoCInt,
    KeyFfi, ReaderUnion, IOX2_OK,
};

use iceoryx2::port::reader::ReaderCreateError;
use iceoryx2::service::port_factory::reader::PortFactoryReader;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_bb_elementary_traits::AsCStr;
use iceoryx2_ffi_macros::{iceoryx2_ffi, CStrRepr};

use core::ffi::{c_char, c_int};
use core::mem::ManuallyDrop;

// BEGIN types definition

#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_reader_create_error_e {
    EXCEEDS_MAX_SUPPORTED_READERS = IOX2_OK as isize + 1,
}

impl IntoCInt for ReaderCreateError {
    fn into_c_int(self) -> c_int {
        (match self {
            ReaderCreateError::ExceedsMaxSupportedReaders => {
                iox2_reader_create_error_e::EXCEEDS_MAX_SUPPORTED_READERS
            }
        }) as c_int
    }
}

pub(super) union PortFactoryReaderBuilderUnion {
    ipc: ManuallyDrop<PortFactoryReader<'static, crate::IpcService, KeyFfi>>,
    local: ManuallyDrop<PortFactoryReader<'static, crate::LocalService, KeyFfi>>,
}

impl PortFactoryReaderBuilderUnion {
    pub(super) fn new_ipc(
        port_factory: PortFactoryReader<'static, crate::IpcService, KeyFfi>,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(port_factory),
        }
    }
    pub(super) fn new_local(
        port_factory: PortFactoryReader<'static, crate::LocalService, KeyFfi>,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(port_factory),
        }
    }
}

#[repr(C)]
#[repr(align(16))] // alignment of Option<PortFactoryReaderBuilderUnion>
pub struct iox2_port_factory_reader_builder_storage_t {
    // TODO: adapt size and alignment
    internal: [u8; 112], // magic number obtained with size_of::<Option<PortFactoryReaderBuilderUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(PortFactoryReaderBuilderUnion)]
pub struct iox2_port_factory_reader_builder_t {
    service_type: iox2_service_type_e,
    value: iox2_port_factory_reader_builder_storage_t,
    deleter: fn(*mut iox2_port_factory_reader_builder_t),
}

impl iox2_port_factory_reader_builder_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: PortFactoryReaderBuilderUnion,
        deleter: fn(*mut iox2_port_factory_reader_builder_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_port_factory_reader_builder_h_t;
/// The owning handle for `iox2_port_factory_reader_builder_t`. Passing the handle to an function transfers the ownership.
pub type iox2_port_factory_reader_builder_h = *mut iox2_port_factory_reader_builder_h_t;
/// The non-owning handle for `iox2_port_factory_reader_builder_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_port_factory_reader_builder_h_ref = *const iox2_port_factory_reader_builder_h;

impl AssertNonNullHandle for iox2_port_factory_reader_builder_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_port_factory_reader_builder_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_port_factory_reader_builder_h {
    type Target = *mut iox2_port_factory_reader_builder_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_port_factory_reader_builder_h_ref {
    type Target = *mut iox2_port_factory_reader_builder_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition

// BEGIN C API

/// Returns a string literal describing the provided [`iox2_reader_create_error_e`].
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
pub unsafe extern "C" fn iox2_reader_create_error_string(
    error: iox2_reader_create_error_e,
) -> *const c_char {
    error.as_const_cstr().as_ptr() as *const c_char
}

/// Creates a reader and consumes the builder
///
/// # Arguments
///
/// * `port_factory_handle` - Must be a valid [`iox2_port_factory_reader_builder_h`] obtained by [`iox2_port_factory_blackboard_reader_builder`](crate::iox2_port_factory_blackboard_reader_builder).
/// * `reader_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_reader_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `reader_handle_ptr` - An uninitialized or dangling [`iox2_reader_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_reader_create_error_e`] otherwise.
///
/// # Safety
///
/// * The `port_factory_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_port_factory_reader_builder_t`]
///   can be re-used with a call to  [`iox2_port_factory_blackboard_reader_builder`](crate::iox2_port_factory_blackboard_reader_builder)!
#[no_mangle]
pub unsafe extern "C" fn iox2_port_factory_reader_builder_create(
    port_factory_handle: iox2_port_factory_reader_builder_h,
    reader_struct_ptr: *mut iox2_reader_t,
    reader_handle_ptr: *mut iox2_reader_h,
) -> c_int {
    debug_assert!(!port_factory_handle.is_null());
    debug_assert!(!reader_handle_ptr.is_null());

    let mut reader_struct_ptr = reader_struct_ptr;
    fn no_op(_: *mut iox2_reader_t) {}
    let mut deleter: fn(*mut iox2_reader_t) = no_op;
    if reader_struct_ptr.is_null() {
        reader_struct_ptr = iox2_reader_t::alloc();
        deleter = iox2_reader_t::dealloc;
    }
    debug_assert!(!reader_struct_ptr.is_null());

    let reader_builder_struct = unsafe { &mut *port_factory_handle.as_type() };
    let service_type = reader_builder_struct.service_type;
    let reader_builder = reader_builder_struct
        .value
        .as_option_mut()
        .take()
        .unwrap_or_else(|| {
            panic!("Trying to use an invalid 'iox2_port_factory_reader_builder_h'!")
        });
    (reader_builder_struct.deleter)(reader_builder_struct);

    match service_type {
        iox2_service_type_e::IPC => {
            let reader_builder = ManuallyDrop::into_inner(reader_builder.ipc);

            match reader_builder.create() {
                Ok(reader) => {
                    (*reader_struct_ptr).init(service_type, ReaderUnion::new_ipc(reader), deleter);
                }
                Err(error) => {
                    return error.into_c_int();
                }
            }
        }
        iox2_service_type_e::LOCAL => {
            let reader_builder = ManuallyDrop::into_inner(reader_builder.local);

            match reader_builder.create() {
                Ok(reader) => {
                    (*reader_struct_ptr).init(
                        service_type,
                        ReaderUnion::new_local(reader),
                        deleter,
                    );
                }
                Err(error) => {
                    return error.into_c_int();
                }
            }
        }
    }

    *reader_handle_ptr = (*reader_struct_ptr).as_handle();

    IOX2_OK
}

// END C API
