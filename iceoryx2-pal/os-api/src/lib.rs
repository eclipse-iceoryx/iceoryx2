// Copyright (c) 2025 Contributors to the Eclipse Foundation
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
#![allow(clippy::missing_safety_doc)]
#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

#[cfg(all(target_os = "linux", not(feature = "libc_platform")))]
#[path = "linux-bindgen/mod.rs"]
pub mod linux;

#[cfg(all(target_os = "linux", feature = "libc_platform"))]
#[path = "linux-libc/mod.rs"]
pub mod linux;

#[cfg(all(not(feature = "libc_platform"), target_os = "linux"))]
pub(crate) mod internal {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]
    #![allow(improper_ctypes)]
    #![allow(unknown_lints)]
    #![allow(unnecessary_transmutes)]
    #![allow(clippy::all)]
    include!(concat!(
        env!("OUT_DIR"),
        env!("BAZEL_BINDGEN_PATH_CORRECTION"),
        "/os_api_generated.rs"
    ));
}
