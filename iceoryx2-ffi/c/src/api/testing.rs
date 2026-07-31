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

use std::os::raw::c_char;

use iceoryx2_bb_posix::{
    config::TEST_DIRECTORY,
    testing::{create_test_directory, generate_file_name, generate_file_path},
};

/// Creates a directory to store test artifacts.
#[unsafe(no_mangle)]
pub extern "C" fn iox2_testing_create_test_directory() {
    create_test_directory();
}

/// Generates a random and all time unique file name.
///
/// # Safety
///
///  * `name` must point to a valid memory location with a capacity of `buffer_len`
///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_testing_generate_file_name(name: *mut c_char, buffer_len: usize) {
    debug_assert!(!name.is_null());
    let file_name = generate_file_name();
    unsafe {
        core::ptr::copy(
            file_name.as_bytes_const().as_ptr(),
            name.cast(),
            file_name.len().min(buffer_len),
        )
    };
}

/// Generates a random and all time unique file name inside the test directory.
///
/// # Safety
///
///  * `name` must point to a valid memory location with a capacity of `buffer_len`
///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_testing_generate_file_path(name: *mut c_char, buffer_len: usize) {
    debug_assert!(!name.is_null());
    let file_path = generate_file_path();
    unsafe {
        core::ptr::copy(
            file_path.as_bytes_const().as_ptr(),
            name.cast(),
            file_path.len().min(buffer_len),
        )
    };
}

/// Returns the test directory path.
///
/// # Safety
///
///  * `name` must point to a valid memory location with a capacity of `buffer_len`
///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iox2_testing_test_directory_path(name: *mut c_char, buffer_len: usize) {
    debug_assert!(!name.is_null());
    unsafe {
        core::ptr::copy(
            TEST_DIRECTORY.as_bytes_const().as_ptr(),
            name.cast(),
            TEST_DIRECTORY.len().min(buffer_len),
        )
    };
}
