// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

#![no_std]

#[cfg(not(target_os = "windows"))]
pub mod settings {
    pub const TEMP_DIRECTORY: &[u8] = b"/tmp/";
    pub const TEST_DIRECTORY: &[u8] = b"/tmp/iceoryx2/tests/";
    pub const SHARED_MEMORY_DIRECTORY: &[u8] = b"/dev/shm/";
    pub const PATH_SEPARATOR: u8 = b'/';
    pub const ROOT: &[u8] = b"/";
    pub const FILENAME_LENGTH: usize = 255;
    pub const PATH_LENGTH: usize = 4096;
    pub const AT_LEAST_TIMING_VARIANCE: f32 = 0.25;
}
#[cfg(not(target_os = "windows"))]
pub use settings::*;

#[cfg(target_os = "windows")]
pub mod settings {
    pub const TEMP_DIRECTORY: &[u8] = b"C:\\Temp\\";
    pub const TEST_DIRECTORY: &[u8] = b"C:\\Temp\\iceoryx2\\tests\\";
    pub const SHARED_MEMORY_DIRECTORY: &[u8] = b"C:\\Temp\\iceoryx2\\shm\\";
    pub const PATH_SEPARATOR: u8 = b'\\';
    pub const ROOT: &[u8] = b"C:\\";
    pub const FILENAME_LENGTH: usize = 255;
    pub const PATH_LENGTH: usize = 255;
    pub const AT_LEAST_TIMING_VARIANCE: f32 = 1.0;
}
#[cfg(target_os = "windows")]
pub use settings::*;
