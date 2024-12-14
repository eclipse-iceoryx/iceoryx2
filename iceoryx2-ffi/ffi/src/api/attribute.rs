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

use crate::api::{
    iox2_callback_context, iox2_event_id_t, iox2_service_type_e, iox2_unique_listener_id_h,
    iox2_unique_listener_id_t, AssertNonNullHandle, HandleToType, IOX2_OK,
};
use crate::iox2_file_descriptor_ptr;

use iceoryx2::service::attribute::Attribute;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_bb_posix::file_descriptor::FileDescriptor;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::ffi::{c_char, c_int};
use core::mem::ManuallyDrop;
use core::time::Duration;

// BEGIN types definition

#[repr(C)]
#[repr(align(8))] // alignment of Attribute
pub struct iox2_attribute_storage_t {
    internal: [u8; 1656], // magic number obtained with size_of::<Attribute>()
}

#[repr(C)]
#[iceoryx2_ffi(Attribute)]
pub struct iox2_attribute_t {
    value: iox2_attribute_storage_t,
    deleter: fn(*mut iox2_attribute_t),
}

impl iox2_attribute_t {
    pub(super) fn init(&mut self, value: Attribute, deleter: fn(*mut iox2_attribute_t)) {
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_attribute_h_t;
/// The owning handle for `iox2_listener_t`. Passing the handle to an function transfers the ownership.
pub type iox2_attribute_h = *mut iox2_attribute_h_t;
/// The non-owning handle for `iox2_listener_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_attribute_h_ref = *const iox2_attribute_h;

impl AssertNonNullHandle for iox2_attribute_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_attribute_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_attribute_h {
    type Target = *mut iox2_attribute_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_attribute_h_ref {
    type Target = *mut iox2_attribute_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}
// END type definition

// BEGIN C API
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_drop(handle: iox2_attribute_h) {
    handle.assert_non_null();

    let attribute = &mut *handle.as_type();

    core::ptr::drop_in_place(attribute.value.as_mut());
    (attribute.deleter)(attribute);
}

#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_key_len(handle: iox2_attribute_h_ref) -> usize {
    handle.assert_non_null();

    let attribute = &mut *handle.as_type();
    attribute.value.as_ref().key().len()
}

#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_key(
    handle: iox2_attribute_h_ref,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    handle.assert_non_null();

    let attribute = &mut *handle.as_type();
    let copied_key_length = buffer_len.min(attribute.value.as_ref().key().len());
    core::ptr::copy_nonoverlapping(
        attribute.value.as_ref().key().as_ptr(),
        buffer.cast(),
        copied_key_length,
    );
    copied_key_length
}

#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_value_len(handle: iox2_attribute_h_ref) -> usize {
    handle.assert_non_null();

    let attribute = &mut *handle.as_type();
    attribute.value.as_ref().value().len()
}

#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_value(
    handle: iox2_attribute_h_ref,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    handle.assert_non_null();

    let attribute = &mut *handle.as_type();
    let copied_key_length = buffer_len.min(attribute.value.as_ref().value().len());
    core::ptr::copy_nonoverlapping(
        attribute.value.as_ref().key().as_ptr(),
        buffer.cast(),
        copied_key_length,
    );
    copied_key_length
}

// END C API
