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

#![allow(non_camel_case_types, non_snake_case)]
#![allow(clippy::missing_safety_doc)]
#![allow(unused_variables)]

use core::unimplemented;

use crate::posix::types::*;

use super::_SC_PAGESIZE;

pub unsafe fn proc_pidpath(pid: pid_t, buffer: *mut c_char, buffer_len: size_t) -> isize {
    0
}

pub unsafe fn sysconf(name: int) -> long {
    if name == _SC_PAGESIZE {
        return 4096;
    }
    -1
}

pub unsafe fn pathconf(path: *const c_char, name: int) -> long {
    unimplemented!("pathconf")
}

pub unsafe fn getpid() -> pid_t {
    0
}

pub unsafe fn getppid() -> pid_t {
    0
}

pub unsafe fn dup(fildes: int) -> int {
    unimplemented!("dup")
}

pub unsafe fn close(fd: int) -> int {
    unimplemented!("close")
}

pub unsafe fn read(fd: int, buf: *mut void, count: size_t) -> ssize_t {
    unimplemented!("read")
}

pub unsafe fn write(fd: int, buf: *const void, count: size_t) -> ssize_t {
    unimplemented!("write")
}

pub unsafe fn access(pathname: *const c_char, mode: int) -> int {
    unimplemented!("access")
}

pub unsafe fn unlink(pathname: *const c_char) -> int {
    unimplemented!("unlink")
}

pub unsafe fn lseek(fd: int, offset: off_t, whence: int) -> off_t {
    unimplemented!("lseek")
}

pub unsafe fn getuid() -> uid_t {
    unimplemented!("getuid")
}

pub unsafe fn getgid() -> gid_t {
    unimplemented!("getgid")
}

pub unsafe fn rmdir(pathname: *const c_char) -> int {
    unimplemented!("rmdir")
}

pub unsafe fn ftruncate(fd: int, length: off_t) -> int {
    unimplemented!("ftruncate")
}

pub unsafe fn fchown(fd: int, owner: uid_t, group: gid_t) -> int {
    unimplemented!("fchown")
}

pub unsafe fn fsync(fd: int) -> int {
    unimplemented!("fsync")
}
