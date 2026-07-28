// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use core::ffi::CStr;
use core::ffi::{c_char, c_int};
use iceoryx2_bb_elementary_traits::AsCStr;
use iceoryx2_bb_flatbuffers::{TypeName, find_best_fitting_schema_file};
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_ffi_macros::CStrRepr;

use crate::IOX2_OK;

#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_flatbuffer_find_schema_file_error_e {
    INVALID_TYPE_NAME_CHARACTERS = IOX2_OK as isize + 1,
    INVALID_TYPE_NAMESPACE_CHARACTERS,
    INVALID_ROOT_PATH,
    BUFFER_TOO_SMALL,
    NO_SCHEMA_FILE_FOUND,
}

/// Generates a random and all time unique file name.
///
/// # Safety
///
///  * `name` must point to a valid memory location with a capacity of `buffer_len`
///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_flatbuffer_find_best_fitting_schema_file(
    type_name: *const c_char,
    type_namespace: *const c_char,
    root_path: *const c_char,
    buffer: *mut c_char,
    buffer_len: usize,
) -> c_int {
    debug_assert!(!type_name.is_null());
    debug_assert!(!root_path.is_null());

    let type_name = match unsafe { CStr::from_ptr(type_name) }.to_str() {
        Ok(v) => v.to_string(),
        Err(_) => {
            return iox2_flatbuffer_find_schema_file_error_e::INVALID_TYPE_NAME_CHARACTERS as _;
        }
    };

    let type_namespace = if type_namespace.is_null() {
        "".to_string()
    } else {
        match unsafe { CStr::from_ptr(type_namespace) }.to_str() {
            Ok(v) => v.to_string(),
            Err(_) => {
                return iox2_flatbuffer_find_schema_file_error_e::INVALID_TYPE_NAMESPACE_CHARACTERS
                    as _;
            }
        }
    };

    let root_path_raw = match unsafe { CStr::from_ptr(root_path) }.to_str() {
        Ok(v) => v.to_string(),
        Err(_) => return iox2_flatbuffer_find_schema_file_error_e::INVALID_ROOT_PATH as _,
    };

    let root_path = match Path::new_normalized(root_path_raw.as_bytes()) {
        Ok(v) => v,
        Err(_) => return iox2_flatbuffer_find_schema_file_error_e::INVALID_ROOT_PATH as _,
    };

    let type_name = TypeName {
        name: unsafe { core::mem::transmute::<&'_ str, &'static str>(type_name.as_str()) },
        namespace: unsafe {
            core::mem::transmute::<&'_ str, &'static str>(type_namespace.as_str())
        },
    };

    let best_fitting_schema = match find_best_fitting_schema_file(&type_name, &root_path) {
        Ok(Some(v)) => v,
        Ok(None) | Err(_) => {
            return iox2_flatbuffer_find_schema_file_error_e::NO_SCHEMA_FILE_FOUND as _;
        }
    };

    if buffer_len < best_fitting_schema.len() {
        return iox2_flatbuffer_find_schema_file_error_e::BUFFER_TOO_SMALL as _;
    }

    unsafe {
        core::ptr::copy(
            best_fitting_schema.as_ptr(),
            buffer.cast(),
            best_fitting_schema.len().min(buffer_len),
        )
    };

    IOX2_OK
}
