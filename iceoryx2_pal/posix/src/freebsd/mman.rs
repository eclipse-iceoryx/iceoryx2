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

#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]

use crate::posix::c_string_length;
use crate::posix::stdlib::*;
use crate::posix::types::*;

pub unsafe fn mlock(addr: *const void, len: size_t) -> int {
    crate::internal::mlock(addr, len)
}

pub unsafe fn munlock(addr: *const void, len: size_t) -> int {
    crate::internal::munlock(addr, len)
}

pub unsafe fn mlockall(flags: int) -> int {
    crate::internal::mlockall(flags)
}

pub unsafe fn munlockall() -> int {
    crate::internal::munlockall()
}

pub unsafe fn shm_open(name: *const char, oflag: int, mode: mode_t) -> int {
    crate::internal::shm_open(name, oflag, mode)
}

pub unsafe fn shm_unlink(name: *const char) -> int {
    crate::internal::shm_unlink(name)
}

pub unsafe fn mmap(
    addr: *mut void,
    len: size_t,
    prot: int,
    flags: int,
    fd: int,
    off: off_t,
) -> *mut void {
    crate::internal::mmap(addr, len, prot, flags, fd, off)
}

pub unsafe fn munmap(addr: *mut void, len: size_t) -> int {
    crate::internal::munmap(addr, len)
}

pub unsafe fn mprotect(addr: *mut void, len: size_t, prot: int) -> int {
    crate::internal::mprotect(addr, len, prot)
}

pub unsafe fn shm_list() -> Vec<[i8; 256]> {
    let mut result = vec![];

    let listmib = b"kern.ipc.posix_shm_list\0";
    let mut mib = [0 as int; 3];
    let mut miblen: size_t = 3;
    if sysctlnametomib(listmib.as_ptr() as *mut i8, mib.as_mut_ptr(), &mut miblen) == -1 {
        return result;
    }

    let mut len: size_t = 0;
    if sysctl(
        mib.as_mut_ptr(),
        miblen as _,
        core::ptr::null_mut::<void>(),
        &mut len,
        core::ptr::null_mut::<void>(),
        0,
    ) == -1
    {
        return result;
    }

    len = len * 4 / 3;
    let buffer = calloc(0, len);

    if buffer.is_null() {
        return result;
    }

    if sysctl(
        mib.as_mut_ptr(),
        miblen as _,
        buffer,
        &mut len,
        core::ptr::null_mut::<void>(),
        0,
    ) != 0
    {
        free(buffer);
        return result;
    }

    let mut temp = buffer;
    let mut current_position = 0;
    while current_position < len {
        let kif = temp as *const kinfo_file;
        if (*kif).kf_structsize == 0 || *(*kif).kf_path.as_ptr() == 0 {
            break;
        }

        let mut name = [0; 256];
        let raw_c = (*kif).kf_path.as_ptr().offset(1);
        let raw_c_len = c_string_length(raw_c);

        name[..raw_c_len].copy_from_slice(core::slice::from_raw_parts(raw_c.cast(), raw_c_len));

        result.push(name);

        temp = temp.add((*kif).kf_structsize as usize);
        current_position += (*kif).kf_structsize as usize;
    }

    free(buffer);
    result
}

pub unsafe fn sysctl(
    name: *mut int,
    namelen: uint,
    oldp: *mut void,
    oldlenp: *mut size_t,
    newp: *mut void,
    newlen: size_t,
) -> int {
    crate::internal::sysctl(name, namelen, oldp, oldlenp, newp, newlen)
}

pub unsafe fn sysctlnametomib(name: *mut char, mibp: *mut int, sizep: *mut size_t) -> int {
    crate::internal::sysctlnametomib(name, mibp, sizep)
}
