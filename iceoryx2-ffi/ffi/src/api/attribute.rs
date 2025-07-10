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
use iceoryx2_bb_container::semantic_string::SemanticString;

use alloc::ffi::CString;
use core::ffi::c_char;

// BEGIN types definition
pub struct iox2_attribute_h_t;

impl iox2_attribute_h_t {
    pub(crate) unsafe fn underlying_type(&self) -> &Attribute {
        &*(self as *const iox2_attribute_h_t).cast()
    }
}

pub type iox2_attribute_h_ref = *const iox2_attribute_h_t;

// END type definition

// BEGIN C API
/// Returns the length of the attributes key.
///
/// # Safety
///
/// * The `handle` must be a valid handle.
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_key_len(handle: iox2_attribute_h_ref) -> usize {
    debug_assert!(!handle.is_null());

    let attribute = (*handle).underlying_type();
    attribute.key().len()
}

/// Copies the keys value into the provided buffer.
///
/// # Safety
///
/// * `handle` - A valid [`iox2_attribute_h_ref`],
/// * `buffer` - Must be non-null and pointing to a valid memory location,
/// * `buffer_len` - Must be the length of the provided `buffer`.
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_key(
    handle: iox2_attribute_h_ref,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    debug_assert!(!handle.is_null());

    let attribute = (*handle).underlying_type();
    if let Ok(key) = CString::new(attribute.key().as_bytes()) {
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

/// Returns the length of the attributes value.
///
/// # Safety
///
/// * The `handle` must be a valid handle.
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_value_len(handle: iox2_attribute_h_ref) -> usize {
    debug_assert!(!handle.is_null());

    let attribute = (*handle).underlying_type();
    attribute.value().len()
}

/// Copies the values value into the provided buffer.
///
/// # Safety
///
/// * `handle` - A valid [`iox2_attribute_h_ref`],
/// * `buffer` - Must be non-null and pointing to a valid memory location,
/// * `buffer_len` - Must be the length of the provided `buffer`.
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_value(
    handle: iox2_attribute_h_ref,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    debug_assert!(!handle.is_null());

    let attribute = (*handle).underlying_type();
    if let Ok(value) = CString::new(attribute.value().as_bytes()) {
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
