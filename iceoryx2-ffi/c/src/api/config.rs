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

use crate::{api::AssertNonNullHandle, c_size_t, iox2_unable_to_deliver_strategy_e};
use core::ffi::{c_char, c_int};
use core::mem::ManuallyDrop;
use core::time::Duration;
use iceoryx2::config::{Config, ConfigCreationError};
use iceoryx2_bb_container::semantic_string::*;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_bb_elementary_traits::AsCStr;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_ffi_macros::iceoryx2_ffi;
use iceoryx2_ffi_macros::CStrRepr;

use crate::IOX2_OK;

use super::{HandleToType, IntoCInt};

// BEGIN type definition

/// Failures occurring while creating a new [`iox2_config_t`] object with [`iox2_config_from_file()`].
#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_config_creation_error_e {
    /// The config file could not be read.
    FAILED_TO_READ_CONFIG_FILE_CONTENTS = IOX2_OK as isize + 1,
    /// Parts of the config file could not be deserialized. Indicates some kind of syntax error.
    UNABLE_TO_DESERIALIZE_CONTENTS,
    /// Insufficient permissions to open the config file.
    INSUFFICIENT_PERMISSIONS,
    /// The provided config file does not exist
    CONFIG_FILE_DOES_NOT_EXIST,
    /// Since the config file could not be opened
    UNABLE_TO_OPEN_CONFIG_FILE,
    /// The provided string is not a valid file path
    INVALID_FILE_PATH,
}

impl IntoCInt for ConfigCreationError {
    fn into_c_int(self) -> c_int {
        (match self {
            ConfigCreationError::FailedToReadConfigFileContents => {
                iox2_config_creation_error_e::FAILED_TO_READ_CONFIG_FILE_CONTENTS
            }
            ConfigCreationError::UnableToDeserializeContents => {
                iox2_config_creation_error_e::UNABLE_TO_DESERIALIZE_CONTENTS
            }
            ConfigCreationError::InsufficientPermissions => {
                iox2_config_creation_error_e::INSUFFICIENT_PERMISSIONS
            }
            ConfigCreationError::ConfigFileDoesNotExist => {
                iox2_config_creation_error_e::CONFIG_FILE_DOES_NOT_EXIST
            }
            ConfigCreationError::UnableToOpenConfigFile => {
                iox2_config_creation_error_e::UNABLE_TO_OPEN_CONFIG_FILE
            }
        }) as c_int
    }
}

// NOTE check the README.md for using opaque types with renaming
/// The immutable pointer to the underlying `Config`
pub type iox2_config_ptr = *const Config;
/// The mutable pointer to the underlying `Config`
pub type iox2_config_ptr_mut = *mut Config;

pub(super) struct ConfigOwner {
    pub(crate) value: ManuallyDrop<Config>,
}

/// A storage object that has the size to store a config
#[repr(C)]
#[repr(align(8))] // align_of<ConfigOwner>()
pub struct iox2_config_storage_t {
    internal: [u8; 4256], // size_of<ConfigOwner>()
}

/// Contains the iceoryx2 config
#[repr(C)]
#[iceoryx2_ffi(ConfigOwner)]
pub struct iox2_config_t {
    pub(crate) value: iox2_config_storage_t,
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

/// The non-owning handle for `iox2_config_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_config_h_ref = *const iox2_config_h;

impl AssertNonNullHandle for iox2_config_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_config_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_config_h {
    type Target = *mut iox2_config_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_config_h_ref {
    type Target = *mut iox2_config_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition

// BEGIN C API

/// Returns a string literal describing the provided [`iox2_config_creation_error_e`].
///
/// # Arguments
///
/// * `error` - The error value for which a description should be returned
///
/// # Returns
///
/// A pointer to a null-terminated string containing the error message.
/// The string is stored in the .rodata section of the binary.
///
/// # Safety
///
/// The returned pointer must not be modified or freed and is valid as long as the program runs.
#[no_mangle]
pub unsafe extern "C" fn iox2_config_creation_error_string(
    error: iox2_config_creation_error_e,
) -> *const c_char {
    error.as_const_cstr().as_ptr() as *const c_char
}

/// This function casts a [`iox2_config_h`] into a [`iox2_config_ptr`]
///
/// # Arguments
///
/// * `handle` obtained by [`iox2_config_from_file()`], [`iox2_config_default()`],
///   [`iox2_config_clone()`] or [`iox2_config_from_ptr()`]
///
/// Returns a [`iox2_config_ptr`]
///
/// # Safety
///
/// * The `config_handle` must be a valid handle.
/// * The `config_handle` is still valid after the call to this function.
#[no_mangle]
pub unsafe extern "C" fn iox2_cast_config_ptr(config_handle: iox2_config_h) -> iox2_config_ptr {
    debug_assert!(!config_handle.is_null());

    &*(*config_handle.as_type()).value.as_ref().value
}

/// Returns a pointer to the global config
#[no_mangle]
pub extern "C" fn iox2_config_global_config() -> iox2_config_ptr {
    iceoryx2::config::Config::global_config()
}

/// Creates an iceoryx2 config populated with default values.
///
/// # Safety
///
/// * `struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_config_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
/// * `handle_ptr` - An uninitialized or dangling [`iox2_config_h`] handle which will be initialized
///   by this function call.
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

/// Creates an iceoryx2 config populated values from the provided file.
///
/// # Safety
///
/// * `struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_config_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
/// * `handle_ptr` - An uninitialized or dangling [`iox2_config_h`] handle which will be initialized
///   by this function call.
/// * `config_file` - Must be a valid file path to an existing config file.
#[no_mangle]
pub unsafe extern "C" fn iox2_config_from_file(
    struct_ptr: *mut iox2_config_t,
    handle_ptr: *mut iox2_config_h,
    config_file: *const c_char,
) -> c_int {
    debug_assert!(!handle_ptr.is_null());
    debug_assert!(!config_file.is_null());

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

/// Clones a config from the provided [`iox2_config_ptr`].
///
/// # Safety
///
/// * `handle` - Must be a valid pointer.
/// * `struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_config_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
/// * `handle_ptr` - An uninitialized or dangling [`iox2_config_h`] handle which will be initialized
///   by this function call.
#[no_mangle]
pub unsafe extern "C" fn iox2_config_from_ptr(
    config: iox2_config_ptr,
    struct_ptr: *mut iox2_config_t,
    handle_ptr: *mut iox2_config_h,
) {
    debug_assert!(!config.is_null());
    debug_assert!(!handle_ptr.is_null());

    let mut struct_ptr = struct_ptr;
    fn no_op(_: *mut iox2_config_t) {}
    let mut deleter: fn(*mut iox2_config_t) = no_op;
    if struct_ptr.is_null() {
        struct_ptr = iox2_config_t::alloc();
        deleter = iox2_config_t::dealloc;
    }
    debug_assert!(!struct_ptr.is_null());

    (*struct_ptr).init(ManuallyDrop::new((*config).clone()), deleter);
    *handle_ptr = (*struct_ptr).as_handle();
}

/// Clones a config from a given non-owning [`iox2_config_h_ref`].
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
/// * `struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_config_t`].
///   If it is a NULL pointer, the storage will be allocated on the heap.
/// * `handle_ptr` - An uninitialized or dangling [`iox2_config_h`] handle which will be initialized
///   by this function call.
#[no_mangle]
pub unsafe extern "C" fn iox2_config_clone(
    handle: iox2_config_h_ref,
    struct_ptr: *mut iox2_config_t,
    handle_ptr: *mut iox2_config_h,
) {
    handle.assert_non_null();
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

/// Takes ownership of the handle and releases all underlying resources.
///
/// # Safety
///
/// * `handle` - An initialized [`iox2_config_h`] handle which will be uninitialized
///   after this function call.
#[no_mangle]
pub unsafe extern "C" fn iox2_config_drop(handle: iox2_config_h) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    ManuallyDrop::drop(&mut config.value.as_mut().value);
    (config.deleter)(config)
}

/////////////////
// BEGIN: global
/////////////////

/// Returns the prefix used for all files created during runtime
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_prefix(handle: iox2_config_h_ref) -> *const c_char {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config.value.as_ref().value.global.prefix.as_c_str()
}

/// Sets the prefix used for all files created during runtime
///
/// Returns: [`iox2_semantic_string_error_e`](crate::api::iox2_semantic_string_error_e) when an
/// invalid file name was provided
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
/// * `value` - A valid file name containing the prefix
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_set_prefix(
    handle: iox2_config_h_ref,
    value: *const c_char,
) -> c_int {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    match FileName::from_c_str(value) {
        Ok(n) => {
            config.value.as_mut().value.global.prefix = n;
            IOX2_OK as _
        }
        Err(e) => e as c_int,
    }
}

/// Returns the path under which all other directories or files will be created
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_root_path(handle: iox2_config_h_ref) -> *const c_char {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config.value.as_ref().value.global.root_path().as_c_str()
}

/// Sets the path under which all other directories or files will be created
///
/// Returns: [`iox2_semantic_string_error_e`](crate::api::iox2_semantic_string_error_e) when an
/// invalid path was provided
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
/// * `value` - A valid path
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_set_root_path(
    handle: iox2_config_h_ref,
    value: *const c_char,
) -> c_int {
    handle.assert_non_null();

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
/// Returns the directory in which all node files are stored
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_directory(
    handle: iox2_config_h_ref,
) -> *const c_char {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config.value.as_ref().value.global.node.directory.as_c_str()
}

/// Sets the directory in which all node files are stored
///
/// Returns: [`iox2_semantic_string_error_e`](crate::api::iox2_semantic_string_error_e) when an
/// invalid path was provided
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
/// * `value` - A valid path
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_set_directory(
    handle: iox2_config_h_ref,
    value: *const c_char,
) -> c_int {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    match Path::from_c_str(value) {
        Ok(n) => {
            config.value.as_mut().value.global.node.directory = n;
            IOX2_OK as _
        }
        Err(e) => e as c_int,
    }
}

/// Returns the suffix of the monitor token
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_monitor_suffix(
    handle: iox2_config_h_ref,
) -> *const c_char {
    handle.assert_non_null();

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

/// Sets the suffix of the monitor token
///
/// Returns: [`iox2_semantic_string_error_e`](crate::api::iox2_semantic_string_error_e) when an
/// invalid file name was provided
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
/// * `value` - A valid file name containing the suffix
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_set_monitor_suffix(
    handle: iox2_config_h_ref,
    value: *const c_char,
) -> c_int {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    match FileName::from_c_str(value) {
        Ok(n) => {
            config.value.as_mut().value.global.node.monitor_suffix = n;
            IOX2_OK as _
        }
        Err(e) => e as c_int,
    }
}

/// Returns the suffix of the files where the node configuration is stored.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_static_config_suffix(
    handle: iox2_config_h_ref,
) -> *const c_char {
    handle.assert_non_null();

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

/// Sets the suffix of the files where the node configuration is stored.
///
/// Returns: [`iox2_semantic_string_error_e`](crate::api::iox2_semantic_string_error_e) when an
/// invalid file name was provided
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
/// * `value` - A valid file name containing the suffix
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_set_static_config_suffix(
    handle: iox2_config_h_ref,
    value: *const c_char,
) -> c_int {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    match FileName::from_c_str(value) {
        Ok(n) => {
            config.value.as_mut().value.global.node.static_config_suffix = n;
            IOX2_OK as _
        }
        Err(e) => e as c_int,
    }
}

/// Returns the suffix of the service tags.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_service_tag_suffix(
    handle: iox2_config_h_ref,
) -> *const c_char {
    handle.assert_non_null();

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

/// Sets the suffix of the service tags.
///
/// Returns: [`iox2_semantic_string_error_e`](crate::api::iox2_semantic_string_error_e) when an
/// invalid file name was provided
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
/// * `value` - A valid file name containing the suffix
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_set_service_tag_suffix(
    handle: iox2_config_h_ref,
    value: *const c_char,
) -> c_int {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    match FileName::from_c_str(value) {
        Ok(n) => {
            config.value.as_mut().value.global.node.service_tag_suffix = n;
            IOX2_OK as _
        }
        Err(e) => e as c_int,
    }
}

/// When true, [`iox2_node_builder_create()`](crate::api::iox2_node_builder_create) checks for dead
/// nodes and cleans up all their stale resources whenever a new
/// [`iox2_node_h`](crate::api::iox2_node_h) is created.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_cleanup_dead_nodes_on_creation(
    handle: iox2_config_h_ref,
) -> bool {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .global
        .node
        .cleanup_dead_nodes_on_creation
}

/// Enable/disable the cleanup dead nodes on creation
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_set_cleanup_dead_nodes_on_creation(
    handle: iox2_config_h_ref,
    value: bool,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .global
        .node
        .cleanup_dead_nodes_on_creation = value;
}

/// When true, the [`iox2_node_builder_create()`](crate::api::iox2_node_builder_create) checks for
/// dead nodes and cleans up all their stale resources whenever an existing
/// [`iox2_node_h`](crate::api::iox2_node_h) is
/// going out of scope.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_cleanup_dead_nodes_on_destruction(
    handle: iox2_config_h_ref,
) -> bool {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .global
        .node
        .cleanup_dead_nodes_on_destruction
}

/// Enable/disable the cleanup dead nodes on destruction
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_node_set_cleanup_dead_nodes_on_destruction(
    handle: iox2_config_h_ref,
    value: bool,
) {
    handle.assert_non_null();

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

/// Returns the directory in which all service files are stored
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_directory(
    handle: iox2_config_h_ref,
) -> *const c_char {
    handle.assert_non_null();

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

/// Sets the directory in which all service files are stored
///
/// Returns: [`iox2_semantic_string_error_e`](crate::api::iox2_semantic_string_error_e) when an
/// invalid path was provided
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
/// * `value` - A valid path
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_set_directory(
    handle: iox2_config_h_ref,
    value: *const c_char,
) -> c_int {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    match Path::from_c_str(value) {
        Ok(n) => {
            config.value.as_mut().value.global.service.directory = n;
            IOX2_OK as _
        }
        Err(e) => e as c_int,
    }
}

/// Returns the suffix of the ports data segment
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_data_segment_suffix(
    handle: iox2_config_h_ref,
) -> *const c_char {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .global
        .service
        .data_segment_suffix
        .as_c_str()
}

/// Sets the suffix of the ports data segment
///
/// Returns: [`iox2_semantic_string_error_e`](crate::api::iox2_semantic_string_error_e) when an
/// invalid file name was provided
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
/// * `value` - A valid file name containing the suffix
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_set_data_segment_suffix(
    handle: iox2_config_h_ref,
    value: *const c_char,
) -> c_int {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    match FileName::from_c_str(value) {
        Ok(n) => {
            config
                .value
                .as_mut()
                .value
                .global
                .service
                .data_segment_suffix = n;
            IOX2_OK as _
        }
        Err(e) => e as c_int,
    }
}

/// Returns the suffix of the static config file
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_static_config_storage_suffix(
    handle: iox2_config_h_ref,
) -> *const c_char {
    handle.assert_non_null();

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

/// Sets the suffix of the static config file
///
/// Returns: [`iox2_semantic_string_error_e`](crate::api::iox2_semantic_string_error_e) when an
/// invalid file name was provided
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
/// * `value` - A valid file name containing the suffix
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_set_static_config_storage_suffix(
    handle: iox2_config_h_ref,
    value: *const c_char,
) -> c_int {
    handle.assert_non_null();

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

/// Returns the suffix of the dynamic config file
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_dynamic_config_storage_suffix(
    handle: iox2_config_h_ref,
) -> *const c_char {
    handle.assert_non_null();

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

/// Sets the suffix of the dynamic config file
///
/// Returns: [`iox2_semantic_string_error_e`](crate::api::iox2_semantic_string_error_e) when an
/// invalid file name was provided
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
/// * `value` - A valid file name containing the suffix
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_set_dynamic_config_storage_suffix(
    handle: iox2_config_h_ref,
    value: *const c_char,
) -> c_int {
    handle.assert_non_null();

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

/// Returns the duration how long another process will wait until the service
/// creation is finalized
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
/// * `secs` - A valid pointer pointing to a [`u64`].
/// * `nsecs` - A valid pointer pointing to a [`u32`]
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_creation_timeout(
    handle: iox2_config_h_ref,
    secs: *mut u64,
    nsecs: *mut u32,
) {
    handle.assert_non_null();
    debug_assert!(!secs.is_null());
    debug_assert!(!nsecs.is_null());

    let config = &*handle.as_type();
    let timeout = config.value.as_ref().value.global.service.creation_timeout;
    *secs = timeout.as_secs();
    *nsecs = timeout.subsec_nanos();
}

/// Sets the creation timeout
///
/// Returns: [`iox2_semantic_string_error_e`](crate::api::iox2_semantic_string_error_e) when an
/// invalid file name was provided
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
/// * `value` - A valid file name containing the suffix
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_set_creation_timeout(
    handle: iox2_config_h_ref,
    sec: u64,
    nsec: u32,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config.value.as_mut().value.global.service.creation_timeout =
        Duration::from_secs(sec) + Duration::from_nanos(nsec as u64);
}

/// The suffix of a one-to-one connection
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_connection_suffix(
    handle: iox2_config_h_ref,
) -> *const c_char {
    handle.assert_non_null();

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

/// Set the suffix of a one-to-one connection
///
/// Returns: [`iox2_semantic_string_error_e`](crate::api::iox2_semantic_string_error_e) when an
/// invalid file name was provided
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
/// * `value` - A valid file name containing the suffix
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_set_connection_suffix(
    handle: iox2_config_h_ref,
    value: *const c_char,
) -> c_int {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    match FileName::from_c_str(value) {
        Ok(n) => {
            config.value.as_mut().value.global.service.connection_suffix = n;
            IOX2_OK as _
        }
        Err(e) => e as c_int,
    }
}

/// Returns the suffix of a one-to-one connection
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_event_connection_suffix(
    handle: iox2_config_h_ref,
) -> *const c_char {
    handle.assert_non_null();

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

/// Sets the suffix of a one-to-one connection
///
/// Returns: [`iox2_semantic_string_error_e`](crate::api::iox2_semantic_string_error_e) when an
/// invalid file name was provided
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
/// * `value` - A valid file name containing the suffix
#[no_mangle]
pub unsafe extern "C" fn iox2_config_global_service_set_event_connection_suffix(
    handle: iox2_config_h_ref,
    value: *const c_char,
) -> c_int {
    handle.assert_non_null();

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
/// Returns the maximum amount of supported [`iox2_subscriber_h`](crate::api::iox2_subscriber_h)s
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_max_subscribers(
    handle: iox2_config_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .publish_subscribe
        .max_subscribers
}

/// Sets the maximum amount of supported [`iox2_subscriber_h`](crate::api::iox2_subscriber_h)s
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_set_max_subscribers(
    handle: iox2_config_h_ref,
    value: c_size_t,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .publish_subscribe
        .max_subscribers = value;
}

/// Returns maximum amount of supported [`iox2_publisher_h`](crate::api::iox2_publisher_h)s
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_max_publishers(
    handle: iox2_config_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .publish_subscribe
        .max_publishers
}

/// Sets the maximum amount of supported [`iox2_publisher_h`](crate::api::iox2_publisher_h)s
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_set_max_publishers(
    handle: iox2_config_h_ref,
    value: c_size_t,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .publish_subscribe
        .max_publishers = value;
}

/// Returns the maximum amount of supported [`iox2_node_h`](crate::api::iox2_node_h)s. Defines indirectly
/// how many processes can open the service at the same time.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_max_nodes(
    handle: iox2_config_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .publish_subscribe
        .max_nodes
}

/// Sets the maximum amount of supported [`iox2_node_h`](crate::api::iox2_node_h)s.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_set_max_nodes(
    handle: iox2_config_h_ref,
    value: c_size_t,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .publish_subscribe
        .max_nodes = value;
}

/// Returns the maximum buffer size a [`iox2_subscriber_h`](crate::api::iox2_subscriber_h) can have
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_subscriber_max_buffer_size(
    handle: iox2_config_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .publish_subscribe
        .subscriber_max_buffer_size
}

/// Sets the maximum buffer size a [`iox2_subscriber_h`](crate::api::iox2_subscriber_h) can have
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_set_subscriber_max_buffer_size(
    handle: iox2_config_h_ref,
    value: c_size_t,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .publish_subscribe
        .subscriber_max_buffer_size = value;
}

/// Returns the maximum amount of [`iox2_sample_h`](crate::api::iox2_sample_h)s a
/// [`iox2_subscriber_h`](crate::api::iox2_subscriber_h) can hold at the same time.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_subscriber_max_borrowed_samples(
    handle: iox2_config_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .publish_subscribe
        .subscriber_max_borrowed_samples
}

/// Sets the maximum amount of [`iox2_sample_h`](crate::api::iox2_sample_h)s a
/// [`iox2_subscriber_h`](crate::api::iox2_subscriber_h) can hold at the same time.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_set_subscriber_max_borrowed_samples(
    handle: iox2_config_h_ref,
    value: c_size_t,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .publish_subscribe
        .subscriber_max_borrowed_samples = value;
}

/// Returns the maximum amount of [`iox2_sample_mut_h`](crate::api::iox2_sample_mut_h)s a
/// [`iox2_publisher_h`](crate::api::iox2_publisher_h) can loan at the same time.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_publisher_max_loaned_samples(
    handle: iox2_config_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .publish_subscribe
        .publisher_max_loaned_samples
}

/// Sets the maximum amount of [`iox2_sample_mut_h`](crate::api::iox2_sample_mut_h)s a
/// [`iox2_publisher_h`](crate::api::iox2_publisher_h) can loan at the same time.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_set_publisher_max_loaned_samples(
    handle: iox2_config_h_ref,
    value: c_size_t,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .publish_subscribe
        .publisher_max_loaned_samples = value;
}

/// Returns the maximum history size a [`iox2_subscriber_h`](crate::api::iox2_subscriber_h) can
/// request from a [`iox2_publisher_h`](crate::api::iox2_publisher_h).
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_publisher_history_size(
    handle: iox2_config_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .publish_subscribe
        .publisher_history_size
}

/// Sets the maximum history size a [`iox2_subscriber_h`](crate::api::iox2_subscriber_h) can
/// request from a [`iox2_publisher_h`](crate::api::iox2_publisher_h).
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_set_publisher_history_size(
    handle: iox2_config_h_ref,
    value: c_size_t,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .publish_subscribe
        .publisher_history_size = value;
}

/// Defines how the [`iox2_subscriber_h`](crate::api::iox2_subscriber_h) buffer behaves when it is
/// full. When safe overflow is activated, the [`iox2_publisher_h`](crate::api::iox2_publisher_h) will
/// replace the oldest [`iox2_sample_h`](crate::api::iox2_sample_h) with the newest one.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_enable_safe_overflow(
    handle: iox2_config_h_ref,
) -> bool {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .publish_subscribe
        .enable_safe_overflow
}

/// Enables/disables safe overflow
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_set_enable_safe_overflow(
    handle: iox2_config_h_ref,
    value: bool,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .publish_subscribe
        .enable_safe_overflow = value;
}

/// If safe overflow is deactivated it defines the deliver strategy of the
/// [`iox2_publisher_h`](crate::api::iox2_publisher_h) when the
/// [`iox2_subscriber_h`](crate::api::iox2_subscriber_h)s buffer is full.
///
/// Returns [`iox2_unable_to_deliver_strategy_e`]
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_unable_to_deliver_strategy(
    handle: iox2_config_h_ref,
) -> c_int {
    handle.assert_non_null();

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

/// Define the unable to deliver strategy
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_set_unable_to_deliver_strategy(
    handle: iox2_config_h_ref,
    value: iox2_unable_to_deliver_strategy_e,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .publish_subscribe
        .unable_to_deliver_strategy = value.into();
}

/// Defines the size of the internal [`iox2_subscriber_h`](crate::api::iox2_subscriber_h)
/// buffer that contains expired connections. An
/// connection is expired when the [`iox2_publisher_h`](crate::api::iox2_publisher_h)
/// disconnected from a service and the connection
/// still contains unconsumed [`iox2_sample_h`](crate::api::iox2_sample_h)s.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_subscriber_expired_connection_buffer(
    handle: iox2_config_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .publish_subscribe
        .subscriber_expired_connection_buffer
}

/// Set the expired connection buffer size
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_publish_subscribe_set_subscriber_expired_connection_buffer(
    handle: iox2_config_h_ref,
    value: c_size_t,
) {
    handle.assert_non_null();

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
// BEGIN: request response
//////////////////////////
/// Returns the expired connection buffer size for [`iox2_client_h`](crate::api::iox2_client_h)
/// to retrieve [`iox2_response_h`](crate::api::iox2_response_h) from disconnected
/// [`iox2_server_h`](crate::api::iox2_server_h)
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_client_expired_connection_buffer(
    handle: iox2_config_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .request_response
        .client_expired_connection_buffer
}

/// Sets the expired connection buffer size for [`iox2_client_h`](crate::api::iox2_client_h).
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_set_client_expired_connection_buffer(
    handle: iox2_config_h_ref,
    value: c_size_t,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .request_response
        .client_expired_connection_buffer = value;
}

/// Returns the expired connection buffer size for [`iox2_serve_h`](crate::api::iox2_server_h)
/// to retrieve [`iox2_active_request_h`](crate::api::iox2_active_request_h) from disconnected
/// [`iox2_client_h`](crate::api::iox2_client_h)
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_server_expired_connection_buffer(
    handle: iox2_config_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .request_response
        .server_expired_connection_buffer
}

/// Sets the expired connection buffer size for [`iox2_server_h`](crate::api::iox2_server_h).
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_set_server_expired_connection_buffer(
    handle: iox2_config_h_ref,
    value: c_size_t,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .request_response
        .server_expired_connection_buffer = value;
}

/// If safe overflow is deactivated it defines the deliver strategy of the
/// [`iox2_client_h`](crate::api::iox2_client_h) when the
/// [`iox2_server_h`](crate::api::iox2_server_h)s request buffer is full.
///
/// Returns [`iox2_unable_to_deliver_strategy_e`]
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_client_unable_to_deliver_strategy(
    handle: iox2_config_h_ref,
) -> c_int {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .request_response
        .client_unable_to_deliver_strategy
        .into_c_int()
}

/// Defines the unable to deliver strategy for the [`iox2_client_h`](crate::api::iox2_client_h).
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_set_client_unable_to_deliver_strategy(
    handle: iox2_config_h_ref,
    value: iox2_unable_to_deliver_strategy_e,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .request_response
        .client_unable_to_deliver_strategy = value.into();
}

/// If safe overflow is deactivated it defines the deliver strategy of the
/// [`iox2_server_h`](crate::api::iox2_server_h) when the
/// [`iox2_client_h`](crate::api::iox2_client_h)s response buffer is full.
///
/// Returns [`iox2_unable_to_deliver_strategy_e`]
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_server_unable_to_deliver_strategy(
    handle: iox2_config_h_ref,
) -> c_int {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .request_response
        .server_unable_to_deliver_strategy
        .into_c_int()
}

/// Defines the unable to deliver strategy for the [`iox2_server_h`](crate::api::iox2_server_h).
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_set_server_unable_to_deliver_strategy(
    handle: iox2_config_h_ref,
    value: iox2_unable_to_deliver_strategy_e,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .request_response
        .server_unable_to_deliver_strategy = value.into();
}

/// Returns if the service supports fire and forget requests. Those are requests where the
/// [`iox2_client_h`](crate::api::iox2_client_h) does not expect a response.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_has_fire_and_forget_requests(
    handle: iox2_config_h_ref,
) -> bool {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .request_response
        .enable_fire_and_forget_requests
}

/// Defines if request response services shall support fire and forget requests.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_set_fire_and_forget_requests(
    handle: iox2_config_h_ref,
    value: bool,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .request_response
        .enable_fire_and_forget_requests = value;
}

/// Defines how the [`iox2_server_h`](crate::api::iox2_server_h) buffer behaves when it is
/// full. When safe overflow is activated, the [`iox2_client_h`](crate::api::iox2_client_h) will
/// replace the oldest [`iox2_request_mut_h`](crate::api::iox2_request_mut_h) with the newest one.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_enable_safe_overflow_for_requests(
    handle: iox2_config_h_ref,
) -> bool {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .request_response
        .enable_safe_overflow_for_requests
}

/// Enables/disables safe overflow for requests
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_set_enable_safe_overflow_for_requests(
    handle: iox2_config_h_ref,
    value: bool,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .request_response
        .enable_safe_overflow_for_requests = value;
}

/// Defines how the [`iox2_client_h`](crate::api::iox2_client_h) buffer behaves when it is
/// full. When safe overflow is activated, the [`iox2_server_h`](crate::api::iox2_server_h) will
/// replace the oldest [`iox2_response_h`](crate::api::iox2_response_h) with the newest one.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_enable_safe_overflow_for_responses(
    handle: iox2_config_h_ref,
) -> bool {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .request_response
        .enable_safe_overflow_for_responses
}

/// Enables/disables safe overflow for responses
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_set_enable_safe_overflow_for_responses(
    handle: iox2_config_h_ref,
    value: bool,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .request_response
        .enable_safe_overflow_for_responses = value;
}

/// Returns how many active requests a [`iox2_client_h`](crate::api::iox2_client_h) can send out in
/// parallel and expect responses from.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_max_active_requests_per_client(
    handle: iox2_config_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .request_response
        .max_active_requests_per_client
}

/// Sets the max number of active requests.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_set_max_active_requests_per_client(
    handle: iox2_config_h_ref,
    value: c_size_t,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .request_response
        .max_active_requests_per_client = value;
}

/// Returns the size of the [`iox2_response_h`](crate::api::iox2_response_h) buffer per request
/// on the [`iox2_client_h`](crate::api::iox2_client_h) side. This is an important setting when
/// a stream of responses is expected.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_max_response_buffer_size(
    handle: iox2_config_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .request_response
        .max_response_buffer_size
}

/// Sets the max response buffer size
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_set_max_response_buffer_size(
    handle: iox2_config_h_ref,
    value: c_size_t,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .request_response
        .max_response_buffer_size = value;
}

/// Returns how many [`iox2_server_h`](crate::api::iox2_server_h)s can be connected to the same
/// service at the same time.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_max_servers(
    handle: iox2_config_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .request_response
        .max_servers
}

/// Sets the maximum number of servers per service
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_set_max_servers(
    handle: iox2_config_h_ref,
    value: c_size_t,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .request_response
        .max_servers = value;
}

/// Returns how many [`iox2_client_h`](crate::api::iox2_client_h)s can be connected to the same
/// service at the same time.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_max_clients(
    handle: iox2_config_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .request_response
        .max_clients
}

/// Sets the maximum number of clients per service
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_set_max_clients(
    handle: iox2_config_h_ref,
    value: c_size_t,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .request_response
        .max_clients = value;
}

/// Returns how many [`iox2_node_h`](crate::api::iox2_node_h)s can open the same
/// service at the same time.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_max_nodes(
    handle: iox2_config_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .request_response
        .max_nodes
}

/// Sets the maximum number of nodes per service
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_set_max_nodes(
    handle: iox2_config_h_ref,
    value: c_size_t,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .request_response
        .max_nodes = value;
}

/// Returns how many [`iox2_response_h`](crate::api::iox2_response_h)s can be borrowed per
/// request.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_max_borrowed_responses_per_pending_response(
    handle: iox2_config_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .request_response
        .max_borrowed_responses_per_pending_response
}

/// Sets the maximum number of borrowed responses per pending response
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_set_max_borrowed_responses_per_pending_response(
    handle: iox2_config_h_ref,
    value: c_size_t,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .request_response
        .max_borrowed_responses_per_pending_response = value;
}

/// Returns how many [`iox2_request_mut_h`](crate::api::iox2_request_mut_h)s can be loaned at most
/// at the same time with a [`iox2_client_h`](crate::api::iox2_client_h).
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_max_loaned_requests(
    handle: iox2_config_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .request_response
        .max_loaned_requests
}

/// Sets the maximum number of loaned requests
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_set_max_loaned_requests(
    handle: iox2_config_h_ref,
    value: c_size_t,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .request_response
        .max_loaned_requests = value
}

/// Returns how many [`iox2_response_mut_h`](crate::api::iox2_response_mut_h)s can be loaned at most
/// at the same time with a [`iox2_server_h`](crate::api::iox2_server_h) per request.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_server_max_loaned_responses_per_request(
    handle: iox2_config_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .request_response
        .server_max_loaned_responses_per_request
}

/// Sets the maximum number of loaned responses per request
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_request_response_set_server_max_loaned_responses_per_request(
    handle: iox2_config_h_ref,
    value: c_size_t,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config
        .value
        .as_mut()
        .value
        .defaults
        .request_response
        .server_max_loaned_responses_per_request = value
}

//////////////////////////
// END: request response
//////////////////////////

//////////////////////////
// BEGIN: event
//////////////////////////
/// Returns the maximum amount of supported [`iox2_listener_h`](crate::api::iox2_listener_h)
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_max_listeners(
    handle: iox2_config_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config.value.as_ref().value.defaults.event.max_listeners
}

/// Sets the maximum amount of supported [`iox2_listener_h`](crate::api::iox2_listener_h)
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_set_max_listeners(
    handle: iox2_config_h_ref,
    value: c_size_t,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config.value.as_mut().value.defaults.event.max_listeners = value;
}

/// Returns the default deadline for event services. If there is a deadline set, the provided
/// arguments `seconds` and `nanoseconds` will be set `true` is returned. Otherwise, false is
/// returned and nothing is set.
///
/// # Safety
///
/// * `notifier_handle` is valid, non-null and was obtained via [`iox2_port_factory_listener_builder_create`](crate::iox2_port_factory_listener_builder_create)
/// * `seconds` is pointing to a valid memory location and non-null
/// * `nanoseconds` is pointing to a valid memory location and non-null
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_deadline(
    handle: iox2_config_h_ref,
    seconds: *mut u64,
    nanoseconds: *mut u32,
) -> bool {
    handle.assert_non_null();
    debug_assert!(!seconds.is_null());
    debug_assert!(!nanoseconds.is_null());

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .event
        .deadline
        .map(|v| {
            *seconds = v.as_secs();
            *nanoseconds = v.subsec_nanos();
        })
        .is_some()
}

/// Sets the default deadline for event services. If `seconds` and `nanoseconds` is `NULL`
/// the deadline will be disabled, otherwise the deadline will be set to the provided values.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
/// * `seconds` & `nanoseconds` - either both must be `NULL` or both must point to a valid memory
///   location
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_set_deadline(
    handle: iox2_config_h_ref,
    seconds: *const u64,
    nanoseconds: *const u32,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    let deadline = if seconds.is_null() {
        debug_assert!(nanoseconds.is_null());
        None
    } else {
        debug_assert!(!nanoseconds.is_null());
        Some(Duration::from_secs(*seconds) + Duration::from_nanos(*nanoseconds as u64))
    };

    config.value.as_mut().value.defaults.event.deadline = deadline;
}

/// Returns the event id value that is emitted when a new notifier is created. It returns `true`
/// if a value is emitted and sets the provided `value`, otherwise it returns `false`.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
/// * `value` - points to a valid memory location
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_notifier_created_event(
    handle: iox2_config_h_ref,
    value: *mut c_size_t,
) -> bool {
    handle.assert_non_null();
    debug_assert!(!value.is_null());

    let config = &*handle.as_type();
    if let Some(v) = config
        .value
        .as_ref()
        .value
        .defaults
        .event
        .notifier_created_event
    {
        *value = v;
        true
    } else {
        false
    }
}

/// Sets the event id value that is emitted when a new notifier is created. If `value` is `NULL`
/// no event will be emitted, otherwise the provided value will be used.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_set_notifier_created_event(
    handle: iox2_config_h_ref,
    value: *const c_size_t,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();

    config
        .value
        .as_mut()
        .value
        .defaults
        .event
        .notifier_created_event = if value.is_null() { None } else { Some(*value) };
}

/// Returns the event id value that is emitted when a notifier is dropped. It returns `true`
/// if a value is emitted and sets the provided `value`, otherwise it returns `false`.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
/// * `value` - points to a valid memory location
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_notifier_dropped_event(
    handle: iox2_config_h_ref,
    value: *mut c_size_t,
) -> bool {
    handle.assert_non_null();
    debug_assert!(!value.is_null());

    let config = &*handle.as_type();
    if let Some(v) = config
        .value
        .as_ref()
        .value
        .defaults
        .event
        .notifier_dropped_event
    {
        *value = v;
        true
    } else {
        false
    }
}

/// Sets the event id value that is emitted when a notifier is dropped. If `value` is `NULL`
/// no event will be emitted, otherwise the provided value will be used.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_set_notifier_dropped_event(
    handle: iox2_config_h_ref,
    value: *const c_size_t,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();

    config
        .value
        .as_mut()
        .value
        .defaults
        .event
        .notifier_dropped_event = if value.is_null() { None } else { Some(*value) };
}

/// Returns the event id value that is emitted when a notifier is identified as dead. It returns
/// `true` if a value is emitted and sets the provided `value`, otherwise it returns `false`.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
/// * `value` - points to a valid memory location
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_notifier_dead_event(
    handle: iox2_config_h_ref,
    value: *mut c_size_t,
) -> bool {
    handle.assert_non_null();
    debug_assert!(!value.is_null());

    let config = &*handle.as_type();
    if let Some(v) = config
        .value
        .as_ref()
        .value
        .defaults
        .event
        .notifier_dead_event
    {
        *value = v;
        true
    } else {
        false
    }
}

/// Sets the event id value that is emitted when a notifier is identified as dead. If `value` is `NULL`
/// no event will be emitted, otherwise the provided value will be used.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_set_notifier_dead_event(
    handle: iox2_config_h_ref,
    value: *const c_size_t,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();

    config
        .value
        .as_mut()
        .value
        .defaults
        .event
        .notifier_dead_event = if value.is_null() { None } else { Some(*value) };
}

/// Returns the maximum amount of supported [`iox2_notifier_h`](crate::api::iox2_notifier_h)
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_max_notifiers(
    handle: iox2_config_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config.value.as_ref().value.defaults.event.max_notifiers
}

/// Sets the maximum amount of supported [`iox2_notifier_h`](crate::api::iox2_notifier_h)
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_set_max_notifiers(
    handle: iox2_config_h_ref,
    value: c_size_t,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config.value.as_mut().value.defaults.event.max_notifiers = value;
}

/// Returns the maximum amount of supported [`iox2_node_h`](crate::api::iox2_node_h)s. Defines
/// indirectly how many processes can open the service at the same time.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_max_nodes(
    handle: iox2_config_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config.value.as_ref().value.defaults.event.max_nodes
}

/// Sets the maximum amount of supported [`iox2_node_h`](crate::api::iox2_node_h)s.
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_set_max_nodes(
    handle: iox2_config_h_ref,
    value: c_size_t,
) {
    handle.assert_non_null();

    let config = &mut *handle.as_type();
    config.value.as_mut().value.defaults.event.max_nodes = value;
}

/// Returns the largest event id supported by the event service
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_event_id_max_value(
    handle: iox2_config_h_ref,
) -> c_size_t {
    handle.assert_non_null();

    let config = &*handle.as_type();
    config
        .value
        .as_ref()
        .value
        .defaults
        .event
        .event_id_max_value
}

/// Sets the largest event id supported by the event service
///
/// # Safety
///
/// * `handle` - A valid non-owning [`iox2_config_h_ref`].
#[no_mangle]
pub unsafe extern "C" fn iox2_config_defaults_event_set_event_id_max_value(
    handle: iox2_config_h_ref,
    value: c_size_t,
) {
    handle.assert_non_null();

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
