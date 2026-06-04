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

#![allow(non_camel_case_types, non_snake_case)]
#![allow(clippy::missing_safety_doc)]

use crate::posix::MemZeroedStruct;
use crate::posix::types::*;

use super::macos_fd_translator::ShmFdTranslator;

pub unsafe fn open_with_mode(pathname: *const c_char, flags: int, mode: mode_t) -> int {
    unsafe { crate::internal::open(pathname, flags, mode as core::ffi::c_uint) }
}

pub unsafe fn fstat(fd: int, buf: *mut stat_t) -> int {
    let mut os_specific_buffer = native_stat_t::new_zeroed();
    let ret = unsafe { crate::internal::fstat(fd, &mut os_specific_buffer) };
    if ret != 0 {
        return ret;
    }

    // iox2-156: mode/uid/gid come from trampoline; size/timestamps from shm.
    if let Some(state_fd) = ShmFdTranslator::get_instance().lookup_state_fd(fd) {
        let user_mode = unsafe { super::mman::read_state_mode(state_fd) };
        os_specific_buffer.st_mode = (os_specific_buffer.st_mode & !0o7777) | (user_mode & 0o7777);

        let mut state_buffer = native_stat_t::new_zeroed();
        if unsafe { crate::internal::fstat(state_fd, &mut state_buffer) } == 0 {
            os_specific_buffer.st_uid = state_buffer.st_uid;
            os_specific_buffer.st_gid = state_buffer.st_gid;
        }
    }

    unsafe { *buf = os_specific_buffer.into() };
    0
}

pub unsafe fn fcntl_int(fd: int, cmd: int, arg: int) -> int {
    unsafe { crate::internal::fcntl(fd, cmd, arg) }
}

pub unsafe fn fcntl(fd: int, cmd: int, arg: *mut flock) -> int {
    unsafe { crate::internal::fcntl(fd, cmd, arg) }
}

pub unsafe fn fcntl2(fd: int, cmd: int) -> int {
    unsafe { crate::internal::fcntl(fd, cmd) }
}

pub unsafe fn fchmod(fd: int, mode: mode_t) -> int {
    // iox2-156: route shm-fd mode change to trampoline content.
    if let Some(state_fd) = ShmFdTranslator::get_instance().lookup_state_fd(fd) {
        if unsafe { super::mman::write_state_mode(state_fd, mode) } {
            0
        } else {
            -1
        }
    } else {
        unsafe { crate::internal::fchmod(fd, mode) }
    }
}

pub unsafe fn open(pathname: *const c_char, flags: int) -> int {
    unsafe { crate::internal::open(pathname, flags) }
}

// iox2-156 regression: fchmod through the wrapper must be visible to fstat.
#[cfg(test)]
mod iox2_156_repro_tests {
    use crate::internal;
    use crate::posix::types::{int, mode_t, stat_t};
    use crate::posix::{Errno, MemZeroedStruct};

    fn unique_shm_name(tag: &str) -> Vec<u8> {
        let pid = unsafe { internal::getpid() };
        format!("/iox2_156_{tag}_{pid}\0").into_bytes()
    }

    #[test]
    fn fchmod_on_shm_fd_should_change_observed_mode() {
        let name = unique_shm_name("verify");
        unsafe { super::super::mman::shm_unlink(name.as_ptr().cast()) };

        let create_mode: mode_t = (internal::S_IRUSR | internal::S_IWUSR) as mode_t;
        let oflag = internal::O_CREAT as int | internal::O_RDWR as int;
        let fd = unsafe { super::super::mman::shm_open(name.as_ptr().cast(), oflag, create_mode) };
        assert!(
            fd >= 0,
            "shm_open failed unexpectedly (errno = {:?})",
            Errno::get()
        );

        let new_mode: mode_t =
            (internal::S_IRUSR | internal::S_IRGRP | internal::S_IROTH) as mode_t;
        let fchmod_ret = unsafe { super::fchmod(fd, new_mode) };

        let mut st = stat_t::new_zeroed();
        let fstat_ret = unsafe { super::fstat(fd, &mut st) };

        unsafe {
            super::super::unistd::close(fd);
            super::super::mman::shm_unlink(name.as_ptr().cast());
        }

        assert_eq!(fchmod_ret, 0, "wrapper fchmod should report success");
        assert_eq!(fstat_ret, 0, "fstat on shm fd should succeed");

        let observed = (st.st_mode as u32) & 0o777;
        let expected = (new_mode as u32) & 0o777;
        assert_eq!(
            observed, expected,
            "fchmod did not propagate to the shm permissions: observed=0o{observed:o}, expected=0o{expected:o}"
        );
    }
}
