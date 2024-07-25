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

use crate::posix::types::*;
use crate::posix::{closedir, free, malloc, opendir, readdir_r};

pub(crate) mod internal {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]
    #![allow(improper_ctypes)]
    #![allow(unknown_lints)]
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/posix_generated.rs"));

    pub const ESUCCES: u32 = 0;
}

#[cfg(target_os = "freebsd")]
mod freebsd;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

pub mod posix {
    #![allow(dead_code)]

    #[cfg(target_os = "freebsd")]
    pub use crate::freebsd::*;
    #[cfg(target_os = "linux")]
    pub use crate::linux::*;
    #[cfg(target_os = "macos")]
    pub use crate::macos::*;
    #[cfg(target_os = "windows")]
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

#[cfg(not(target_os = "windows"))]
pub unsafe fn scandir_impl(
    path: *const c_char,
    namelist: *mut *mut *mut crate::posix::types::dirent,
) -> int {
    let dirfd = opendir(path);
    if dirfd.is_null() {
        return -1;
    }

    *namelist = core::ptr::null_mut::<*mut dirent>();
    let mut entries = vec![];
    const DIRENT_SIZE: usize = core::mem::size_of::<dirent>();

    let cleanup = |entries: &Vec<*mut void>, namelist: *mut *mut dirent| {
        for entry in entries {
            free((*entry).cast());
        }

        if !namelist.is_null() {
            free(namelist.cast());
        }
    };

    loop {
        let dirent_ptr = malloc(DIRENT_SIZE);
        let result_ptr: *mut *mut dirent = malloc(core::mem::size_of::<*mut dirent>()).cast();

        if readdir_r(dirfd, dirent_ptr.cast(), result_ptr) != 0 {
            free(result_ptr.cast());
            free(dirent_ptr);
            cleanup(&entries, *namelist);

            closedir(dirfd);
            return -1;
        }

        if (*result_ptr).is_null() {
            free(result_ptr.cast());
            free(dirent_ptr);
            break;
        }

        free(result_ptr.cast());
        entries.push(dirent_ptr);
    }

    *namelist = malloc(core::mem::size_of::<*mut *mut dirent>() * entries.len()).cast();
    if (*namelist).is_null() {
        cleanup(&entries, *namelist);
        closedir(dirfd);
        return -1;
    }

    for (n, entry) in entries.iter().enumerate() {
        (*namelist).add(n).write((*entry).cast());
    }

    closedir(dirfd);
    entries.len() as _
}

#[cfg(target_os = "windows")]
pub(crate) mod win_internal {
    #![allow(dead_code)]
    use std::os::windows::prelude::OsStrExt;

    pub(crate) unsafe fn print_char(value: *const crate::posix::c_char) {
        let len = crate::posix::c_string_length(value);

        let text =
            std::str::from_utf8(core::slice::from_raw_parts(value as *const u8, len)).unwrap();
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
