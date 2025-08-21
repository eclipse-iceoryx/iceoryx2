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

use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_bb_posix::{
    file_descriptor::{FileDescriptor, FileDescriptorBased},
    file_descriptor_set::SynchronousMultiplexing,
};
use iceoryx2_ffi_macros::iceoryx2_ffi;

use super::{AssertNonNullHandle, HandleToType};

// BEGIN type definition

#[derive(Debug)]
#[repr(C)]
pub struct CFileDescriptor {
    value: i32,
    is_owned: bool,
}

impl FileDescriptorBased for CFileDescriptor {
    fn file_descriptor(&self) -> &FileDescriptor {
        unsafe { core::mem::transmute(self) }
    }
}

impl SynchronousMultiplexing for CFileDescriptor {}

#[repr(C)]
#[repr(align(8))] // alignment of Option<CFileDescriptor>
pub struct iox2_file_descriptor_storage_t {
    internal: [u8; 8], // magic number obtained with size_of::<Option<CFileDescriptor>>()
}

#[repr(C)]
#[iceoryx2_ffi(CFileDescriptor)]
pub struct iox2_file_descriptor_t {
    pub(crate) value: iox2_file_descriptor_storage_t,
    deleter: fn(*mut iox2_file_descriptor_t),
}

impl iox2_file_descriptor_t {
    pub(super) fn init(
        &mut self,
        value: CFileDescriptor,
        deleter: fn(*mut iox2_file_descriptor_t),
    ) {
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_file_descriptor_h_t;
/// The owning handle for `iox2_file_descriptor_t`. Passing the handle to an function transfers the ownership.
pub type iox2_file_descriptor_h = *mut iox2_file_descriptor_h_t;

/// The non-owning handle for `iox2_file_descriptor_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_file_descriptor_h_ref = *const iox2_file_descriptor_h;

// NOTE check the README.md for using opaque types with renaming
/// The immutable pointer to the underlying `FileDescriptor`
pub type iox2_file_descriptor_ptr = *const CFileDescriptor;
/// The mutable pointer to the underlying `FileDescriptor`
pub type iox2_file_descriptor_ptr_mut = *mut CFileDescriptor;

impl AssertNonNullHandle for iox2_file_descriptor_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_file_descriptor_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_file_descriptor_h {
    type Target = *mut iox2_file_descriptor_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_file_descriptor_h_ref {
    type Target = *mut iox2_file_descriptor_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}
// END type definition

// BEGIN C API

/// Casts a [`iox2_file_descriptor_h`] into an [`iox2_file_descriptor_ptr`]. The result
/// is valid as long as the source is valid.
///
/// # Safety
///
/// * `handle` must be valid and acquired with [`iox2_file_descriptor_new()`].
#[no_mangle]
pub unsafe extern "C" fn iox2_cast_file_descriptor_ptr(
    handle: iox2_file_descriptor_h,
) -> iox2_file_descriptor_ptr {
    debug_assert!(!handle.is_null());

    (*handle.as_type()).value.as_ref()
}

/// Releases a [`iox2_file_descriptor_h`].
///
/// # Safety
///
/// * `handle` must be valid and acquired with [`iox2_file_descriptor_new()`].
#[no_mangle]
pub unsafe extern "C" fn iox2_file_descriptor_drop(handle: iox2_file_descriptor_h) {
    handle.assert_non_null();

    let file_descriptor = &mut *handle.as_type();
    core::ptr::drop_in_place(file_descriptor.value.as_option_mut());
    (file_descriptor.deleter)(file_descriptor);
}

/// Returns the underlying native file descriptor value. When the
/// [`iox2_file_descriptor_h_ref`] is owning the file descriptor, the native value is
/// valid until [`iox2_file_descriptor_drop()`] is called.
///
/// # Safety
///
/// * `handle` must be valid and acquired with [`iox2_file_descriptor_new()`].
#[no_mangle]
pub unsafe extern "C" fn iox2_file_descriptor_native_handle(
    handle: iox2_file_descriptor_ptr,
) -> i32 {
    debug_assert!(!handle.is_null());
    (*handle).file_descriptor().native_handle()
}

/// Creates a new [`iox2_file_descriptor_t`].
///
/// # Return
///
/// Returns true, when the [`iox2_file_descriptor_h`] was initialized successfully.
/// If the user provided an invalid file descriptor it returns false.
///
/// # Safety
///
/// * `struct_ptr` must be either null or pointing to a valid uninitialized memory location
/// * `handle_ptr` must be non-null and pointing to valid uninitialized memory
/// * `handle_ptr` must be cleaned up with [`iox2_file_descriptor_drop()`]
#[no_mangle]
pub unsafe extern "C" fn iox2_file_descriptor_new(
    value: i32,
    is_owned: bool,
    struct_ptr: *mut iox2_file_descriptor_t,
    handle_ptr: *mut iox2_file_descriptor_h,
) -> bool {
    debug_assert!(!handle_ptr.is_null());

    if FileDescriptor::non_owning_new(value).is_none() {
        return false;
    }

    let mut struct_ptr = struct_ptr;
    fn no_op(_: *mut iox2_file_descriptor_t) {}
    let mut deleter: fn(*mut iox2_file_descriptor_t) = no_op;
    if struct_ptr.is_null() {
        struct_ptr = iox2_file_descriptor_t::alloc();
        deleter = iox2_file_descriptor_t::dealloc;
    }
    debug_assert!(!struct_ptr.is_null());

    (*struct_ptr).init(CFileDescriptor { value, is_owned }, deleter);
    *handle_ptr = (*struct_ptr).as_handle();

    true
}
// END C API
