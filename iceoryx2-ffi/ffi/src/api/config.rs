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

use core::ffi::{c_char, c_int};
use iceoryx2::config::{Config, ConfigCreationError};
use iceoryx2_bb_container::semantic_string::*;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_ffi_macros::iceoryx2_ffi;
use std::mem::ManuallyDrop;

use crate::IOX2_OK;

use super::{HandleToType, IntoCInt};

// BEGIN type definition
#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_config_creation_error_e {
    FAILED_TO_OPEN_CONFIG_FILE = IOX2_OK as isize + 1,
    FAILED_TO_READ_CONFIG_FILE_CONTENTS,
    UNABLE_TO_DESERIALIZE_CONTENTS,
    INVALID_FILE_PATH,
}

impl IntoCInt for ConfigCreationError {
    fn into_c_int(self) -> c_int {
        (match self {
            ConfigCreationError::FailedToOpenConfigFile => {
                iox2_config_creation_error_e::FAILED_TO_OPEN_CONFIG_FILE
            }
            ConfigCreationError::FailedToReadConfigFileContents => {
                iox2_config_creation_error_e::FAILED_TO_READ_CONFIG_FILE_CONTENTS
            }
            ConfigCreationError::UnableToDeserializeContents => {
                iox2_config_creation_error_e::UNABLE_TO_DESERIALIZE_CONTENTS
            }
        }) as c_int
    }
}

// NOTE check the README.md for using opaque types with renaming
/// The immutable pointer to the underlying `Config`
pub type iox2_config_ptr = *const Config;
/// The mutable pointer to the underlying `Config`
pub type iox2_config_mut_ptr = *mut Config;

pub(super) struct ConfigOwner {
    value: ManuallyDrop<Config>,
}

#[repr(C)]
#[repr(align(8))]
pub struct iox2_config_storage_t {
    internal: [u8; 3560],
}

#[repr(C)]
#[iceoryx2_ffi(ConfigOwner)]
pub struct iox2_config_t {
    value: iox2_config_storage_t,
    deleter: fn(*mut iox2_config_t),
}

impl iox2_config_t {
    pub(super) fn init(&mut self, value: ManuallyDrop<Config>, deleter: fn(*mut iox2_config_t)) {
        self.value.init(ConfigOwner { value });
        self.deleter = deleter;
    }
}

pub struct iox2_config_h_t;
/// The owning handle for `iox2_config_t`. Passing the handle to an function transfers the ownership.
pub type iox2_config_h = *mut iox2_config_h_t;

pub struct iox2_config_ref_h_t;
/// The non-owning handle for `iox2_config_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_config_ref_h = *mut iox2_config_ref_h_t;

impl HandleToType for iox2_config_h {
    type Target = *mut iox2_config_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_config_ref_h {
    type Target = *mut iox2_config_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

// END type definition

// BEGIN C API
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_config() -> iox2_config_ptr {
    iceoryx2::config::Config::global_config()
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_from_file(
    handle: iox2_config_ref_h,
    struct_ptr: *mut iox2_config_t,
    handle_ptr: *mut iox2_config_h,
    config_file: *const c_char,
) -> c_int {
    debug_assert!(!handle.is_null());
    debug_assert!(!handle_ptr.is_null());

    let file = match FilePath::from_c_str(config_file) {
        Ok(file) => file,
        Err(_) => return iox2_config_creation_error_e::INVALID_FILE_PATH as c_int,
    };

    let mut struct_ptr = struct_ptr;
    fn no_op(_: *mut iox2_config_t) {}
    let mut deleter: fn(*mut iox2_config_t) = no_op;
    if struct_ptr.is_null() {
        struct_ptr = iox2_config_t::alloc();
        deleter = iox2_config_t::dealloc;
    }
    debug_assert!(!struct_ptr.is_null());

    let config_from_file = match Config::from_file(&file) {
        Ok(config) => config,
        Err(e) => {
            deleter(struct_ptr);
            return e.into_c_int();
        }
    };

    (*struct_ptr).init(ManuallyDrop::new(config_from_file), deleter);

    IOX2_OK
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_clone(
    handle: iox2_config_ref_h,
    struct_ptr: *mut iox2_config_t,
    handle_ptr: *mut iox2_config_h,
) {
    debug_assert!(!handle.is_null());
    debug_assert!(!handle_ptr.is_null());

    let mut struct_ptr = struct_ptr;
    fn no_op(_: *mut iox2_config_t) {}
    let mut deleter: fn(*mut iox2_config_t) = no_op;
    if struct_ptr.is_null() {
        struct_ptr = iox2_config_t::alloc();
        deleter = iox2_config_t::dealloc;
    }
    debug_assert!(!struct_ptr.is_null());

    let config = &*handle.as_type();

    (*struct_ptr).init(config.value.as_ref().value.clone(), deleter);
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_drop(handle: iox2_config_h) {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    ManuallyDrop::drop(&mut config.value.as_mut().value);
    (config.deleter)(config)
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_prefix(handle: iox2_config_ref_h) -> *const c_char {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config.value.as_ref().value.global.prefix.as_c_str()
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_set_prefix(
    handle: iox2_config_ref_h,
    value: *const c_char,
) -> c_int {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    match FileName::from_c_str(value) {
        Ok(n) => {
            config.value.as_mut().value.global.prefix = n;
            IOX2_OK as _
        }
        Err(e) => e as c_int,
    }
}

// END C API
