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

use crate::{c_size_t, iox2_unable_to_deliver_strategy_e};
use core::ffi::{c_char, c_int};
use core::time::Duration;
use iceoryx2::config::{Config, ConfigCreationError};
use iceoryx2_bb_container::semantic_string::*;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_system_types::path::Path;
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
pub unsafe extern "C" fn iox2_cast_config_ref_h(handle: iox2_config_h) -> iox2_config_ref_h {
    debug_assert!(!handle.is_null());

    (*handle.as_type()).as_ref_handle() as *mut _ as _
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_config() -> iox2_config_ptr {
    iceoryx2::config::Config::global_config()
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_default(
    struct_ptr: *mut iox2_config_t,
    handle_ptr: *mut iox2_config_h,
) -> c_int {
    debug_assert!(!handle_ptr.is_null());

    let mut struct_ptr = struct_ptr;
    fn no_op(_: *mut iox2_config_t) {}
    let mut deleter: fn(*mut iox2_config_t) = no_op;
    if struct_ptr.is_null() {
        struct_ptr = iox2_config_t::alloc();
        deleter = iox2_config_t::dealloc;
    }
    debug_assert!(!struct_ptr.is_null());

    (*struct_ptr).init(ManuallyDrop::new(Config::default()), deleter);
    *handle_ptr = (*struct_ptr).as_handle();

    IOX2_OK
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_from_file(
    struct_ptr: *mut iox2_config_t,
    handle_ptr: *mut iox2_config_h,
    config_file: *const c_char,
) -> c_int {
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
    *handle_ptr = (*struct_ptr).as_handle();

    IOX2_OK
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_from_ptr(
    handle: iox2_config_ptr,
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

    (*struct_ptr).init(ManuallyDrop::new((*handle).clone()), deleter);
    *handle_ptr = (*struct_ptr).as_handle();
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

    let config = &mut *handle.as_type();
    (*struct_ptr).init(config.value.as_ref().value.clone(), deleter);
    *handle_ptr = (*struct_ptr).as_handle();
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_drop(handle: iox2_config_h) {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    ManuallyDrop::drop(&mut config.value.as_mut().value);
    (config.deleter)(config)
}

/////////////////
// BEGIN: global
/////////////////

#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_prefix(handle: iox2_config_ref_h) -> *const c_char {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config.value.as_ref().value.global.prefix.as_c_str()
}

/// Returns: iox2_semantic_string_error
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

#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_root_path(handle: iox2_config_ref_h) -> *const c_char {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config.value.as_ref().value.global.root_path().as_c_str()
}

/// Returns: iox2_semantic_string_error
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_set_root_path(
    handle: iox2_config_ref_h,
    value: *const c_char,
) -> c_int {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    match Path::from_c_str(value) {
        Ok(n) => {
            config.value.as_mut().value.global.set_root_path(&n);
            IOX2_OK as _
        }
        Err(e) => e as c_int,
    }
}

/////////////////
// END: global
/////////////////

/////////////////
// BEGIN: node
/////////////////
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_directory(
    handle: iox2_config_ref_h,
) -> *const c_char {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config.value.as_ref().value.global.node.directory.as_c_str()
}

/// Returns: iox2_semantic_string_error
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_set_directory(
    handle: iox2_config_ref_h,
    value: *const c_char,
) -> c_int {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    match Path::from_c_str(value) {
        Ok(n) => {
            config.value.as_mut().value.global.node.directory = n;
            IOX2_OK as _
        }
        Err(e) => e as c_int,
    }
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_monitor_suffix(
    handle: iox2_config_ref_h,
) -> *const c_char {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .global
        .node
        .monitor_suffix
        .as_c_str()
}

/// Returns: iox2_semantic_string_error
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_set_monitor_suffix(
    handle: iox2_config_ref_h,
    value: *const c_char,
) -> c_int {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    match FileName::from_c_str(value) {
        Ok(n) => {
            config.value.as_mut().value.global.node.monitor_suffix = n;
            IOX2_OK as _
        }
        Err(e) => e as c_int,
    }
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_static_config_suffix(
    handle: iox2_config_ref_h,
) -> *const c_char {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .global
        .node
        .static_config_suffix
        .as_c_str()
}

/// Returns: iox2_semantic_string_error
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_set_static_config_suffix(
    handle: iox2_config_ref_h,
    value: *const c_char,
) -> c_int {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    match FileName::from_c_str(value) {
        Ok(n) => {
            config.value.as_mut().value.global.node.static_config_suffix = n;
            IOX2_OK as _
        }
        Err(e) => e as c_int,
    }
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_service_tag_suffix(
    handle: iox2_config_ref_h,
) -> *const c_char {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .global
        .node
        .service_tag_suffix
        .as_c_str()
}

/// Returns: iox2_semantic_string_error
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_set_service_tag_suffix(
    handle: iox2_config_ref_h,
    value: *const c_char,
) -> c_int {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    match FileName::from_c_str(value) {
        Ok(n) => {
            config.value.as_mut().value.global.node.service_tag_suffix = n;
            IOX2_OK as _
        }
        Err(e) => e as c_int,
    }
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_cleanup_dead_nodes_on_creation(
    handle: iox2_config_ref_h,
) -> bool {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .global
        .node
        .cleanup_dead_nodes_on_creation
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_set_cleanup_dead_nodes_on_creation(
    handle: iox2_config_ref_h,
    value: bool,
) {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .global
        .node
        .cleanup_dead_nodes_on_creation = value;
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_cleanup_dead_nodes_on_destruction(
    handle: iox2_config_ref_h,
) -> bool {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .global
        .node
        .cleanup_dead_nodes_on_destruction
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_set_cleanup_dead_nodes_on_destruction(
    handle: iox2_config_ref_h,
    value: bool,
) {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .global
        .node
        .cleanup_dead_nodes_on_destruction = value;
}

/////////////////
// END: node
/////////////////

/////////////////
// BEGIN: service
/////////////////
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_directory(
    handle: iox2_config_ref_h,
) -> *const c_char {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .global
        .service
        .directory
        .as_c_str()
}

/// Returns: iox2_semantic_string_error
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_set_directory(
    handle: iox2_config_ref_h,
    value: *const c_char,
) -> c_int {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    match Path::from_c_str(value) {
        Ok(n) => {
            config.value.as_mut().value.global.service.directory = n;
            IOX2_OK as _
        }
        Err(e) => e as c_int,
    }
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_publisher_data_segment_suffix(
    handle: iox2_config_ref_h,
) -> *const c_char {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .global
        .service
        .publisher_data_segment_suffix
        .as_c_str()
}

/// Returns: iox2_semantic_string_error
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_set_publisher_data_segment_suffix(
    handle: iox2_config_ref_h,
    value: *const c_char,
) -> c_int {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    match FileName::from_c_str(value) {
        Ok(n) => {
            config
                .value
                .as_mut()
                .value
                .global
                .service
                .publisher_data_segment_suffix = n;
            IOX2_OK as _
        }
        Err(e) => e as c_int,
    }
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_static_config_storage_suffix(
    handle: iox2_config_ref_h,
) -> *const c_char {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .global
        .service
        .static_config_storage_suffix
        .as_c_str()
}

/// Returns: iox2_semantic_string_error
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_set_static_config_storage_suffix(
    handle: iox2_config_ref_h,
    value: *const c_char,
) -> c_int {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    match FileName::from_c_str(value) {
        Ok(n) => {
            config
                .value
                .as_mut()
                .value
                .global
                .service
                .static_config_storage_suffix = n;
            IOX2_OK as _
        }
        Err(e) => e as c_int,
    }
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_dynamic_config_storage_suffix(
    handle: iox2_config_ref_h,
) -> *const c_char {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .global
        .service
        .dynamic_config_storage_suffix
        .as_c_str()
}

/// Returns: iox2_semantic_string_error
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_set_dynamic_config_storage_suffix(
    handle: iox2_config_ref_h,
    value: *const c_char,
) -> c_int {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    match FileName::from_c_str(value) {
        Ok(n) => {
            config
                .value
                .as_mut()
                .value
                .global
                .service
                .dynamic_config_storage_suffix = n;
            IOX2_OK as _
        }
        Err(e) => e as c_int,
    }
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_creation_timeout_sec(
    handle: iox2_config_ref_h,
) -> u64 {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .global
        .service
        .creation_timeout
        .as_secs()
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_creation_timeout_nsec_frac(
    handle: iox2_config_ref_h,
) -> u32 {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .global
        .service
        .creation_timeout
        .subsec_nanos()
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_set_creation_timeout(
    handle: iox2_config_ref_h,
    sec: u64,
    nsec: u32,
) {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    config.value.as_mut().value.global.service.creation_timeout =
        Duration::from_secs(sec) + Duration::from_nanos(nsec as u64);
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_connection_suffix(
    handle: iox2_config_ref_h,
) -> *const c_char {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .global
        .service
        .connection_suffix
        .as_c_str()
}

/// Returns: iox2_semantic_string_error
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_set_connection_suffix(
    handle: iox2_config_ref_h,
    value: *const c_char,
) -> c_int {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    match FileName::from_c_str(value) {
        Ok(n) => {
            config.value.as_mut().value.global.service.connection_suffix = n;
            IOX2_OK as _
        }
        Err(e) => e as c_int,
    }
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_event_connection_suffix(
    handle: iox2_config_ref_h,
) -> *const c_char {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .global
        .service
        .event_connection_suffix
        .as_c_str()
}

/// Returns: iox2_semantic_string_error
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_set_event_connection_suffix(
    handle: iox2_config_ref_h,
    value: *const c_char,
) -> c_int {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    match FileName::from_c_str(value) {
        Ok(n) => {
            config
                .value
                .as_mut()
                .value
                .global
                .service
                .event_connection_suffix = n;
            IOX2_OK as _
        }
        Err(e) => e as c_int,
    }
}
/////////////////
// END: service
/////////////////

//////////////////////////
// BEGIN: publish subscribe
//////////////////////////
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_max_subscribers(
    handle: iox2_config_ref_h,
) -> c_size_t {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .publish_subscribe
        .max_subscribers
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_set_max_subscribers(
    handle: iox2_config_ref_h,
    value: c_size_t,
) {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .publish_subscribe
        .max_subscribers = value;
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_max_publishers(
    handle: iox2_config_ref_h,
) -> c_size_t {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .publish_subscribe
        .max_publishers
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_set_max_publishers(
    handle: iox2_config_ref_h,
    value: c_size_t,
) {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .publish_subscribe
        .max_publishers = value;
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_max_nodes(
    handle: iox2_config_ref_h,
) -> c_size_t {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .publish_subscribe
        .max_nodes
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_set_max_nodes(
    handle: iox2_config_ref_h,
    value: c_size_t,
) {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .publish_subscribe
        .max_nodes = value;
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_subscriber_max_buffer_size(
    handle: iox2_config_ref_h,
) -> c_size_t {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .publish_subscribe
        .subscriber_max_buffer_size
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_set_subscriber_max_buffer_size(
    handle: iox2_config_ref_h,
    value: c_size_t,
) {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .publish_subscribe
        .subscriber_max_buffer_size = value;
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_subscriber_max_borrowed_samples(
    handle: iox2_config_ref_h,
) -> c_size_t {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .publish_subscribe
        .subscriber_max_borrowed_samples
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_set_subscriber_max_borrowed_samples(
    handle: iox2_config_ref_h,
    value: c_size_t,
) {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .publish_subscribe
        .subscriber_max_borrowed_samples = value;
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_publisher_max_loaned_samples(
    handle: iox2_config_ref_h,
) -> c_size_t {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .publish_subscribe
        .publisher_max_loaned_samples
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_set_publisher_max_loaned_samples(
    handle: iox2_config_ref_h,
    value: c_size_t,
) {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .publish_subscribe
        .publisher_max_loaned_samples = value;
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_publisher_history_size(
    handle: iox2_config_ref_h,
) -> c_size_t {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .publish_subscribe
        .publisher_history_size
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_set_publisher_history_size(
    handle: iox2_config_ref_h,
    value: c_size_t,
) {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .publish_subscribe
        .publisher_history_size = value;
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_enable_safe_overflow(
    handle: iox2_config_ref_h,
) -> bool {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .publish_subscribe
        .enable_safe_overflow
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_set_enable_safe_overflow(
    handle: iox2_config_ref_h,
    value: bool,
) {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .publish_subscribe
        .enable_safe_overflow = value;
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_unable_to_deliver_strategy(
    handle: iox2_config_ref_h,
) -> c_int {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .publish_subscribe
        .unable_to_deliver_strategy
        .into_c_int()
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_set_unable_to_deliver_strategy(
    handle: iox2_config_ref_h,
    value: iox2_unable_to_deliver_strategy_e,
) {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .publish_subscribe
        .unable_to_deliver_strategy = value.into();
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_subscriber_expired_connection_buffer(
    handle: iox2_config_ref_h,
) -> c_size_t {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .publish_subscribe
        .subscriber_expired_connection_buffer
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_set_subscriber_expired_connection_buffer(
    handle: iox2_config_ref_h,
    value: c_size_t,
) {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .publish_subscribe
        .subscriber_expired_connection_buffer = value;
}
//////////////////////////
// END: publish subscribe
//////////////////////////

//////////////////////////
// BEGIN: event
//////////////////////////
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_max_listeners(
    handle: iox2_config_ref_h,
) -> c_size_t {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config.value.as_ref().value.defaults.event.max_listeners
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_set_max_listeners(
    handle: iox2_config_ref_h,
    value: c_size_t,
) {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    config.value.as_mut().value.defaults.event.max_listeners = value;
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_max_notifiers(
    handle: iox2_config_ref_h,
) -> c_size_t {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config.value.as_ref().value.defaults.event.max_notifiers
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_set_max_notifiers(
    handle: iox2_config_ref_h,
    value: c_size_t,
) {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    config.value.as_mut().value.defaults.event.max_notifiers = value;
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_max_nodes(
    handle: iox2_config_ref_h,
) -> c_size_t {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config.value.as_ref().value.defaults.event.max_nodes
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_set_max_nodes(
    handle: iox2_config_ref_h,
    value: c_size_t,
) {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    config.value.as_mut().value.defaults.event.max_nodes = value;
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_event_id_max_value(
    handle: iox2_config_ref_h,
) -> c_size_t {
    debug_assert!(!handle.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .event
        .event_id_max_value
}

#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_set_event_id_max_value(
    handle: iox2_config_ref_h,
    value: c_size_t,
) {
    debug_assert!(!handle.is_null());

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .event
        .event_id_max_value = value;
}
//////////////////////////
// END: event
//////////////////////////
// END C API
