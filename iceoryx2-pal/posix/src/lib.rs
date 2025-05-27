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

#![allow(clippy::missing_safety_doc)]
#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

mod common;

#[cfg(not(feature = "libc_platform"))]
pub(crate) mod internal {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]
    #![allow(improper_ctypes)]
    #![allow(unknown_lints)]
    #![allow(clippy::all)]
    include!(concat!(
        env!("OUT_DIR"),
        env!("BAZEL_BINDGEN_PATH_CORRECTION"),
        "/posix_generated.rs"
    ));

    pub const ESUCCES: u32 = 0;
}

#[cfg(feature = "libc_platform")]
mod libc;

#[cfg(all(target_os = "freebsd", not(feature = "libc_platform")))]
mod freebsd;
#[cfg(all(target_os = "linux", not(feature = "libc_platform")))]
mod linux;
#[cfg(all(target_os = "macos", not(feature = "libc_platform")))]
mod macos;
#[cfg(all(target_os = "windows", not(feature = "libc_platform")))]
mod windows;

pub mod posix {
    #![allow(dead_code)]
    use super::*;

    pub use common::cpu_set_t::cpu_set_t;
    pub use common::mem_zeroed_struct::MemZeroedStruct;
    pub use common::sockaddr_in::SockAddrIn;
    pub(crate) use common::string_operations::*;

    #[cfg(feature = "libc_platform")]
    pub use crate::libc::*;

    #[cfg(all(target_os = "freebsd", not(feature = "libc_platform")))]
    pub use crate::freebsd::*;
    #[cfg(all(target_os = "linux", not(feature = "libc_platform")))]
    pub use crate::linux::*;
    #[cfg(all(target_os = "macos", not(feature = "libc_platform")))]
    pub use crate::macos::*;
    #[cfg(all(target_os = "windows", not(feature = "libc_platform")))]
    pub use crate::windows::*;
}
