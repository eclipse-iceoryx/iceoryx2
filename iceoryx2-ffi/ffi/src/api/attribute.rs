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

use iceoryx2::service::attribute::Attribute;

use core::ffi::c_char;

// BEGIN types definition

pub type iox2_attribute_h_ref = *const Attribute;

// END type definition

// BEGIN C API
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_key_len(handle: iox2_attribute_h_ref) -> usize {
    debug_assert!(!handle.is_null());

    let attribute = &*handle;
    attribute.key().len()
}

#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_key(
    handle: iox2_attribute_h_ref,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    debug_assert!(!handle.is_null());

    let attribute = &*handle;
    let copied_key_length = buffer_len.min(attribute.key().len());
    core::ptr::copy_nonoverlapping(attribute.key().as_ptr(), buffer.cast(), copied_key_length);
    copied_key_length
}

#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_value_len(handle: iox2_attribute_h_ref) -> usize {
    debug_assert!(!handle.is_null());

    let attribute = &*handle;
    attribute.value().len()
}

#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_value(
    handle: iox2_attribute_h_ref,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    debug_assert!(!handle.is_null());

    let attribute = &*handle;
    let copied_key_length = buffer_len.min(attribute.value().len());
    core::ptr::copy_nonoverlapping(attribute.key().as_ptr(), buffer.cast(), copied_key_length);
    copied_key_length
}

// END C API
