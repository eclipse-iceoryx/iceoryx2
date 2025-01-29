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

#[cfg(not(target_os = "windows"))]
mod scandir;
#[cfg(not(target_os = "windows"))]
use scandir::*;

pub mod posix {
    #![allow(dead_code)]

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

    pub trait Struct: Sized {
        fn new() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[repr(C)]
    pub struct cpu_set_t {
        pub __bits: [u8; CPU_SETSIZE / 8],
    }
    impl Struct for cpu_set_t {}

    pub trait SockAddrIn {
        fn set_s_addr(&mut self, value: u32);
        fn get_s_addr(&self) -> u32;
    }

    impl cpu_set_t {
        pub fn set(&mut self, cpu: usize) {
            if cpu > CPU_SETSIZE {
                return;
            }

            let index = cpu / 8;
            let offset = cpu % 8;

            self.__bits[index] |= 1 << offset;
        }

        pub fn has(&self, cpu: usize) -> bool {
            if cpu > CPU_SETSIZE {
                return false;
            }

            let index = cpu / 8;
            let offset = cpu % 8;
            self.__bits[index] & (1 << offset) != 0
        }

        pub(crate) fn new_allow_all() -> Self {
            Self {
                __bits: [0xff; CPU_SETSIZE / 8],
            }
        }
    }

    pub(crate) unsafe fn c_string_length(value: *const crate::posix::c_char) -> usize {
        for i in 0..isize::MAX {
            if *value.offset(i) == crate::posix::NULL_TERMINATOR {
                return i as usize;
            }
        }

        unreachable!()
    }
}

#[cfg(all(target_os = "windows", not(feature = "libc_platform")))]
pub(crate) mod win_internal {
    #![allow(dead_code)]
    use std::os::windows::prelude::OsStrExt;

    pub(crate) unsafe fn print_char(value: *const crate::posix::c_char) {
        let len = crate::posix::c_string_length(value);

        let text =
            core::str::from_utf8(core::slice::from_raw_parts(value as *const u8, len)).unwrap();
        println!("{}", text);
    }

    pub(crate) unsafe fn c_wide_string_length(value: *const u16) -> usize {
        for i in 0..usize::MAX {
            if *value.add(i) == 0u16 {
                return i;
            }
        }

        0
    }

    pub(crate) unsafe fn c_string_to_wide_string(value: *const crate::posix::c_char) -> Vec<u16> {
        let value_str = core::str::from_utf8_unchecked(core::slice::from_raw_parts(
            value as *const u8,
            crate::posix::c_string_length(value),
        ));
        let mut result: Vec<u16> = std::ffi::OsStr::new(value_str).encode_wide().collect();
        result.push(0);
        result
    }
}
