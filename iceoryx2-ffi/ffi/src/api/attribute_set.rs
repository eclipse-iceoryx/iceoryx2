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

use std::ffi::{c_char, CStr, CString};

use iceoryx2::service::attribute::{Attribute, AttributeSet};
use iceoryx2_bb_elementary::CallbackProgression;

use super::{iox2_attribute_h_ref, iox2_callback_context, iox2_callback_progression_e};

// BEGIN types definition
pub struct iox2_attribute_set_h_t;

impl iox2_attribute_set_h_t {
    pub(crate) unsafe fn underlying_type(&self) -> &AttributeSet {
        &*(self as *const iox2_attribute_set_h_t).cast()
    }
}

pub type iox2_attribute_set_h_ref = *const iox2_attribute_set_h_t;

pub type iox2_attribute_set_get_callback =
    extern "C" fn(*const c_char, iox2_callback_context) -> iox2_callback_progression_e;

// END type definition

// BEGIN C API

/// Returns the length of the attribute set.
///
/// # Safety
///
/// * The `handle` must be a valid handle.
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_set_len(handle: iox2_attribute_set_h_ref) -> usize {
    debug_assert!(!handle.is_null());

    let attribute_set = (*handle).underlying_type();
    attribute_set.iter().len()
}

/// Returns a [`iox2_attribute_h_ref`] to the attribute stored at the provided index.
///
/// # Safety
///
/// * The `handle` must be a valid handle.
/// * The `index` < [`iox2_attribute_set_len()`].
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_set_at(
    handle: iox2_attribute_set_h_ref,
    index: usize,
) -> iox2_attribute_h_ref {
    debug_assert!(!handle.is_null());
    debug_assert!(index < iox2_attribute_set_len(handle));

    let attribute_set = (*handle).underlying_type();
    (&attribute_set[index] as *const Attribute).cast()
}

/// Calls the provided callback for every value that is owned by the provided key.
///
/// # Safety
///
/// * The `handle` must be a valid handle.
/// * The `key` must be a valid null-terminated string.
/// * The `callback` must point to a function with the required signature.
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_set_get_key_values(
    handle: iox2_attribute_set_h_ref,
    key: *const c_char,
    callback: iox2_attribute_set_get_callback,
    callback_ctx: iox2_callback_context,
) {
    debug_assert!(!handle.is_null());

    let attribute_set = (*handle).underlying_type();
    let key = CStr::from_ptr(key);
    let c_str = key.to_str();
    if c_str.is_err() {
        return;
    }

    let c_str = c_str.unwrap();

    attribute_set.get_key_values(c_str, |value| {
        if let Ok(value) = CString::new(value) {
            callback(value.as_ptr(), callback_ctx).into()
        } else {
            CallbackProgression::Continue
        }
    });
}
// END C API
