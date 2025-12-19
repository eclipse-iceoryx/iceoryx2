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

#![cfg_attr(
    not(any(
        target_os = "android",
        target_os = "windows",
        target_os = "macos",
        target_os = "freebsd"
    )),
    no_std
)]
#![allow(clippy::missing_safety_doc)]
#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

extern crate alloc;

mod common;

#[cfg(platform_override)]
mod os {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]
    #![allow(improper_ctypes)]
    #![allow(unknown_lints)]
    #![allow(unnecessary_transmutes)]
    #![allow(clippy::all)]
    include!(concat!(env!("IOX2_CUSTOM_POSIX_PLATFORM_PATH"), "/os.rs"));
}

#[cfg(not(platform_override))]
#[cfg(all(target_os = "linux", feature = "libc_platform"))]
#[path = "libc/os.rs"]
mod os;

#[cfg(not(platform_override))]
#[cfg(target_os = "android")]
#[path = "android/os.rs"]
mod os;

#[cfg(not(platform_override))]
#[cfg(target_os = "freebsd")]
#[path = "freebsd/os.rs"]
mod os;

#[cfg(not(platform_override))]
#[cfg(target_os = "macos")]
#[path = "macos/os.rs"]
mod os;

#[cfg(not(platform_override))]
#[cfg(all(target_os = "linux", not(feature = "libc_platform")))]
#[path = "linux/os.rs"]
pub mod os;

#[cfg(not(platform_override))]
#[cfg(target_os = "nto")]
#[path = "qnx/os.rs"]
mod os;

#[cfg(not(platform_override))]
#[cfg(target_os = "windows")]
#[path = "windows/os.rs"]
mod os;

#[cfg(not(platform_override))]
#[cfg(all(target_os = "none", not(feature = "libc_platform")))]
#[path = "stub/os.rs"]
mod os;

#[cfg(all(not(feature = "libc_platform"), not(target_os = "none")))]
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
        "/posix_generated.rs"
    ));

    pub const ESUCCES: u32 = 0;
}

#[cfg(feature = "libc_platform")]
pub(crate) mod internal {
    pub use libc::*;
}

pub mod posix {
    #![allow(dead_code)]
    use super::*;

    pub use common::cpu_set_t::cpu_set_t;
    pub use common::mem_zeroed_struct::MemZeroedStruct;
    pub use common::sockaddr_in::SockAddrIn;

    #[allow(unused_imports)]
    pub(crate) use common::string_operations::*;

    pub use crate::os::posix::*;
}
