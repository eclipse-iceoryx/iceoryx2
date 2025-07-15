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

use crate::posix::types::*;
use alloc::ffi::CString;

pub unsafe fn proc_pidpath(pid: pid_t, buffer: *mut c_char, buffer_len: size_t) -> isize {
    let path = if pid == crate::internal::getpid() {
        c"/proc/self/exe".to_owned()
    } else {
        CString::new(format!("/proc/{pid}/exe")).expect("String without 0 bytes")
    };
    crate::internal::readlink(path.as_ptr().cast(), buffer.cast(), buffer_len)
}

pub unsafe fn sysconf(name: int) -> long {
    crate::internal::sysconf(name)
}

pub unsafe fn pathconf(path: *const c_char, name: int) -> long {
    crate::internal::pathconf(path, name)
}

pub unsafe fn getpid() -> pid_t {
    crate::internal::getpid()
}

pub unsafe fn getppid() -> pid_t {
    crate::internal::getppid()
}

pub unsafe fn dup(fildes: int) -> int {
    crate::internal::dup(fildes)
}

pub unsafe fn close(fd: int) -> int {
    crate::internal::close(fd)
}

pub unsafe fn read(fd: int, buf: *mut void, count: size_t) -> ssize_t {
    crate::internal::read(fd, buf, count)
}

pub unsafe fn write(fd: int, buf: *const void, count: size_t) -> ssize_t {
    crate::internal::write(fd, buf, count)
}

pub unsafe fn access(pathname: *const c_char, mode: int) -> int {
    crate::internal::access(pathname, mode)
}

pub unsafe fn unlink(pathname: *const c_char) -> int {
    crate::internal::unlink(pathname)
}

pub unsafe fn lseek(fd: int, offset: off_t, whence: int) -> off_t {
    crate::internal::lseek(fd, offset, whence)
}

pub unsafe fn getuid() -> uid_t {
    crate::internal::getuid()
}

pub unsafe fn getgid() -> gid_t {
    crate::internal::getgid()
}

pub unsafe fn rmdir(pathname: *const c_char) -> int {
    crate::internal::rmdir(pathname)
}

pub unsafe fn ftruncate(fd: int, length: off_t) -> int {
    crate::internal::ftruncate(fd, length)
}

pub unsafe fn fchown(fd: int, owner: uid_t, group: gid_t) -> int {
    crate::internal::fchown(fd, owner, group)
}

pub unsafe fn fsync(fd: int) -> int {
    crate::internal::fsync(fd)
}
