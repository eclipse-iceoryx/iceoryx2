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

#[cfg(all(not(target_os = "windows"), not(target_os = "nto")))]
pub mod settings {
    pub const GLOBAL_CONFIG_PATH: &[u8] = b"/etc";
    pub const USER_CONFIG_PATH: &[u8] = b".config";
    pub const TEMP_DIRECTORY: &[u8] = b"/tmp/";
    pub const TEST_DIRECTORY: &[u8] = b"/tmp/iceoryx2/tests/";
    pub const SHARED_MEMORY_DIRECTORY: &[u8] = b"/dev/shm/";
    pub const PATH_SEPARATOR: u8 = b'/';
    pub const ROOT: &[u8] = b"/";
    pub const ICEORYX2_ROOT_PATH: &[u8] = b"/tmp/iceoryx2/";
    pub const FILENAME_LENGTH: usize = 255;
    // it is actually 4096 but to be more compatible with windows and also safe some stack the number
    // is reduced to 255
    pub const PATH_LENGTH: usize = 255;
    #[cfg(not(target_os = "macos"))]
    pub const AT_LEAST_TIMING_VARIANCE: f32 = 0.25;
    #[cfg(target_os = "macos")]
    pub const AT_LEAST_TIMING_VARIANCE: f32 = 1.0;
}

#[cfg(target_os = "nto")]
pub mod settings {
    pub const GLOBAL_CONFIG_PATH: &[u8] = b"/etc";
    pub const USER_CONFIG_PATH: &[u8] = b".config";
    pub const TEMP_DIRECTORY: &[u8] = b"/data/iceoryx2/tmp/";
    pub const TEST_DIRECTORY: &[u8] = b"/data/iceoryx2/tests/";
    pub const SHARED_MEMORY_DIRECTORY: &[u8] = b"/dev/shmem/";
    pub const PATH_SEPARATOR: u8 = b'/';
    pub const ROOT: &[u8] = b"/";
    pub const ICEORYX2_ROOT_PATH: &[u8] = b"/data/iceoryx2/";
    pub const FILENAME_LENGTH: usize = 255;
    pub const PATH_LENGTH: usize = 255;
    pub const AT_LEAST_TIMING_VARIANCE: f32 = 0.25;
}

#[cfg(target_os = "windows")]
pub mod settings {
    pub const GLOBAL_CONFIG_PATH: &[u8] = b"C:\\ProgramData";
    pub const USER_CONFIG_PATH: &[u8] = b".config";
    pub const TEMP_DIRECTORY: &[u8] = b"C:\\Temp\\";
    pub const TEST_DIRECTORY: &[u8] = b"C:\\Temp\\iceoryx2\\tests\\";
    pub const SHARED_MEMORY_DIRECTORY: &[u8] = b"C:\\Temp\\iceoryx2\\shm\\";
    pub const PATH_SEPARATOR: u8 = b'\\';
    pub const ROOT: &[u8] = b"C:\\";
    pub const ICEORYX2_ROOT_PATH: &[u8] = b"C:\\Temp\\iceoryx2\\";
    pub const FILENAME_LENGTH: usize = 255;
    pub const PATH_LENGTH: usize = 255;
    pub const AT_LEAST_TIMING_VARIANCE: f32 = 1.0;
}

pub use settings::*;
