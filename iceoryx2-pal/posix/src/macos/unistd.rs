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

use crate::posix::constants::*;
use crate::posix::settings::*;
use crate::posix::types::*;

pub unsafe fn proc_pidpath(pid: pid_t, buffer: *mut c_char, buffer_len: size_t) -> isize {
    unsafe {
        if libc::getpid() == pid as _ {
            let mut size: u32 = buffer_len as u32;
            #[allow(deprecated)]
            // _NSGetExecutablePath needs to be replaced with functionality from the 'mach2' crate
            if libc::_NSGetExecutablePath(buffer.cast(), &mut size) != 0 {
                return -1;
            }

            return libc::strnlen(buffer.cast(), buffer_len) as _;
        }

        let ret_val = libc::proc_pidpath(pid as _, buffer.cast(), buffer_len as _);
        if ret_val <= 0 {
            return -1;
        }

        libc::strnlen(buffer.cast(), buffer_len) as _
    }
}

pub unsafe fn sysconf(name: int) -> long {
    unsafe { libc::sysconf(name) }
}

pub unsafe fn pathconf(path: *const c_char, name: int) -> long {
    if name == _PC_NAME_MAX {
        return MAX_FILE_NAME_LENGTH as _;
    }

    unsafe { libc::pathconf(path, name) }
}

pub unsafe fn getpid() -> pid_t {
    unsafe { libc::getpid() }
}

pub unsafe fn gethostpid() -> pid_t {
    unsafe { libc::getpid() }
}

pub unsafe fn getppid() -> pid_t {
    unsafe { libc::getppid() }
}

pub unsafe fn dup(fildes: int) -> int {
    unsafe { libc::dup(fildes) }
}

pub unsafe fn close(fd: int) -> int {
    // iox2-156: unregister before close to avoid fd-reuse races.
    let state_fd = super::macos_fd_translator::ShmFdTranslator::get_instance().unregister(fd);
    let ret = unsafe { libc::close(fd) };
    if let Some(state_fd) = state_fd {
        unsafe { libc::close(state_fd) };
    }
    ret
}

pub unsafe fn read(fd: int, buf: *mut void, count: size_t) -> ssize_t {
    unsafe { libc::read(fd, buf, count) }
}

pub unsafe fn write(fd: int, buf: *const void, count: size_t) -> ssize_t {
    unsafe { libc::write(fd, buf, count) }
}

pub unsafe fn access(pathname: *const c_char, mode: int) -> int {
    unsafe { libc::access(pathname, mode) }
}

pub unsafe fn unlink(pathname: *const c_char) -> int {
    unsafe { libc::unlink(pathname) }
}

pub unsafe fn lseek(fd: int, offset: off_t, whence: int) -> off_t {
    unsafe { libc::lseek(fd, offset, whence) }
}

pub unsafe fn getuid() -> uid_t {
    unsafe { libc::getuid() }
}

pub unsafe fn getgid() -> gid_t {
    unsafe { libc::getgid() }
}

pub unsafe fn rmdir(pathname: *const c_char) -> int {
    unsafe { libc::rmdir(pathname) }
}

pub unsafe fn ftruncate(fd: int, length: off_t) -> int {
    unsafe { libc::ftruncate(fd, length) }
}

pub unsafe fn fchown(fd: int, owner: uid_t, group: gid_t) -> int {
    // iox2-156: ownership lives on the trampoline state file for shm fds.
    let target_fd = super::macos_fd_translator::ShmFdTranslator::get_instance()
        .lookup_state_fd(fd)
        .unwrap_or(fd);
    unsafe { libc::fchown(target_fd, owner, group) }
}

pub unsafe fn fsync(fd: int) -> int {
    unsafe { libc::fsync(fd) }
}
