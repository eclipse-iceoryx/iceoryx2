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

use crate::posix::{closedir, opendir, readdir, types::*};

pub unsafe fn mlock(addr: *const void, len: size_t) -> int {
    libc::mlock(addr, len)
}

pub unsafe fn munlock(addr: *const void, len: size_t) -> int {
    libc::munlock(addr, len)
}

pub unsafe fn mlockall(flags: int) -> int {
    libc::mlockall(flags)
}

pub unsafe fn munlockall() -> int {
    libc::munlockall()
}

pub unsafe fn shm_open(name: *const c_char, oflag: int, mode: mode_t) -> int {
    libc::shm_open(name, oflag, mode)
}

pub unsafe fn shm_unlink(name: *const c_char) -> int {
    libc::shm_unlink(name)
}

pub unsafe fn shm_list() -> Vec<[i8; 256]> {
    let mut result = vec![];
    let dir = opendir(c"/dev/shm/".as_ptr().cast());
    if dir.is_null() {
        return result;
    }

    loop {
        let entry = readdir(dir);
        if entry.is_null() {
            break;
        }
        let mut temp = [0i8; 256];
        for (i, c) in temp.iter_mut().enumerate() {
            *c = (*entry).d_name[i] as _;
            if (*entry).d_name[i] == 0 {
                break;
            }
        }

        // skip empty names
        if temp[0] == 0 ||
        // skip dot (for current dir)
        temp[0] as u8 == b'.' && temp[1] == 0 ||
        // skip  dot dot (for parent dir)
        temp[0] as u8 == b'.' && temp[1] as u8 == b'.' && temp[2] == 0
        {
            continue;
        }

        result.push(temp);
    }
    closedir(dir);

    result
}

pub unsafe fn mmap(
    addr: *mut void,
    len: size_t,
    prot: int,
    flags: int,
    fd: int,
    off: off_t,
) -> *mut void {
    libc::mmap(addr, len, prot, flags, fd, off)
}

pub unsafe fn munmap(addr: *mut void, len: size_t) -> int {
    libc::munmap(addr, len)
}

pub unsafe fn mprotect(addr: *mut void, len: size_t, prot: int) -> int {
    libc::mprotect(addr, len, prot)
}
