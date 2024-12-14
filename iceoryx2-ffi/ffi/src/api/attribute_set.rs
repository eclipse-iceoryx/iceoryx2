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

use iceoryx2::service::attribute::AttributeSet;
use iceoryx2_bb_elementary::CallbackProgression;

use super::{iox2_attribute_h_ref, iox2_callback_context, iox2_callback_progression_e};

// BEGIN types definition

pub type iox2_attribute_set_h_ref = *const AttributeSet;

pub type iox2_attribute_set_get_callback =
    extern "C" fn(*const c_char, iox2_callback_context) -> iox2_callback_progression_e;

// END type definition

// BEGIN C API
#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_set_len(handle: iox2_attribute_set_h_ref) -> usize {
    debug_assert!(!handle.is_null());

    let attribute_set = &*handle;
    attribute_set.iter().len()
}

#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_set_at(
    handle: iox2_attribute_set_h_ref,
    index: usize,
) -> iox2_attribute_h_ref {
    debug_assert!(!handle.is_null());
    debug_assert!(index < iox2_attribute_set_len(handle));

    let attribute_set = &*handle;
    &attribute_set[index]
}

#[no_mangle]
pub unsafe extern "C" fn iox2_attribute_set_get_key_values(
    handle: iox2_attribute_set_h_ref,
    key: *const c_char,
    callback: iox2_attribute_set_get_callback,
    callback_ctx: iox2_callback_context,
) {
    debug_assert!(!handle.is_null());

    let attribute_set = &*handle;
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
