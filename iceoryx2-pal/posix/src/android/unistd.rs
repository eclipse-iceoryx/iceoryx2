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
    let path = if pid == libc::getpid() {
        c"/proc/self/exe".to_owned()
    } else {
        CString::new(format!("/proc/{pid}/exe")).expect("String without 0 bytes")
    };
    libc::readlink(path.as_bytes().as_ptr().cast(), buffer.cast(), buffer_len)
}

pub unsafe fn sysconf(name: int) -> long {
    libc::sysconf(name)
}

pub unsafe fn pathconf(path: *const c_char, name: int) -> long {
    libc::pathconf(path, name)
}

pub unsafe fn getpid() -> pid_t {
    libc::getpid()
}

pub unsafe fn getppid() -> pid_t {
    libc::getppid()
}

pub unsafe fn dup(fildes: int) -> int {
    libc::dup(fildes)
}

pub unsafe fn close(fd: int) -> int {
    libc::close(fd)
}

pub unsafe fn read(fd: int, buf: *mut void, count: size_t) -> ssize_t {
    libc::read(fd, buf, count)
}

pub unsafe fn write(fd: int, buf: *const void, count: size_t) -> ssize_t {
    libc::write(fd, buf, count)
}

pub unsafe fn access(pathname: *const c_char, mode: int) -> int {
    libc::access(pathname, mode)
}

pub unsafe fn unlink(pathname: *const c_char) -> int {
    libc::unlink(pathname)
}

pub unsafe fn lseek(fd: int, offset: off_t, whence: int) -> off_t {
    libc::lseek(fd, offset, whence)
}

pub unsafe fn getuid() -> uid_t {
    libc::getuid()
}

pub unsafe fn getgid() -> gid_t {
    libc::getgid()
}

pub unsafe fn rmdir(pathname: *const c_char) -> int {
    libc::rmdir(pathname)
}

pub unsafe fn ftruncate(fd: int, length: off_t) -> int {
    libc::ftruncate(fd, length)
}

pub unsafe fn fchown(fd: int, owner: uid_t, group: gid_t) -> int {
    libc::fchown(fd, owner, group)
}

pub unsafe fn fsync(fd: int) -> int {
    libc::fsync(fd)
}
