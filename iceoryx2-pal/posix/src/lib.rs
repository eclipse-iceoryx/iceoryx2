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

// Need to use target instead of `std` flag to support commands that build
// crates in isolation, such as:
//   cargo build --workspace --all-targets
//
// While depending purely on the `std` feature flag here would be more
// consistent, such commands seem to only build with default features,
// and crates do not have `std` enabled by default to simplify `no_std` builds.
#![cfg_attr(
    any(target_os = "linux", target_os = "nto", target_os = "none"),
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
#[cfg(target_os = "linux")]
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
#[cfg(target_os = "none")]
#[path = "stub/os.rs"]
mod os;

#[cfg(platform_binding = "bindgen")]
pub(crate) mod internal {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]
    #![allow(improper_ctypes)]
    #![allow(unknown_lints)]
    #![allow(unnecessary_transmutes)]
    #![allow(clippy::all)]
    #[cfg(not(bazel_build))]
    include!(concat!(env!("OUT_DIR"), "/posix_generated.rs"));

    #[cfg(bazel_build)]
    pub use iceoryx2_pal_posix_bindgen::*;

    pub const ESUCCES: u32 = 0;
}

#[cfg(platform_binding = "libc")]
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
