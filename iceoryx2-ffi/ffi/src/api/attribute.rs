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
use std::ffi::CString;

// BEGIN types definition

pub type iox2_attribute_h_ref = *const Attribute;

// END type definition

// BEGIN C API
const ZERO_TERMINATOR_LEN: usize = 1;

#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_key_len(handle: iox2_attribute_h_ref) -> usize {
    debug_assert!(!handle.is_null());

    let attribute = &*handle;
    attribute.key().len() + ZERO_TERMINATOR_LEN
}

#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_key(
    handle: iox2_attribute_h_ref,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    debug_assert!(!handle.is_null());

    let attribute = &*handle;
    if let Ok(key) = CString::new(attribute.key()) {
        let copied_key_length = buffer_len.min(key.as_bytes_with_nul().len());
        core::ptr::copy_nonoverlapping(
            key.as_bytes_with_nul().as_ptr(),
            buffer.cast(),
            copied_key_length,
        );
        copied_key_length
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_value_len(handle: iox2_attribute_h_ref) -> usize {
    debug_assert!(!handle.is_null());

    let attribute = &*handle;
    attribute.value().len() + ZERO_TERMINATOR_LEN
}

#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_value(
    handle: iox2_attribute_h_ref,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    debug_assert!(!handle.is_null());

    let attribute = &*handle;
    if let Ok(value) = CString::new(attribute.value()) {
        let copied_key_length = buffer_len.min(value.as_bytes_with_nul().len());
        core::ptr::copy_nonoverlapping(
            value.as_bytes_with_nul().as_ptr(),
            buffer.cast(),
            copied_key_length,
        );
        copied_key_length
    } else {
        0
    }
}

// END C API
