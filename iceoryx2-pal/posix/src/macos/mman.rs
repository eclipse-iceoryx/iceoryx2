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

use iceoryx2_pal_settings::PATH_SEPARATOR;

use crate::posix::*;

use super::{
    open_with_mode,
    settings::{MAX_PATH_LENGTH, SHM_STATE_DIRECTORY, SHM_STATE_SUFFIX},
};

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

unsafe fn remove_leading_path_separator(value: *const char) -> *const char {
    if *value as u8 == PATH_SEPARATOR {
        value.add(1)
    } else {
        value
    }
}

unsafe fn shm_file_path(name: *const char, suffix: &[u8]) -> [u8; MAX_PATH_LENGTH] {
    let name = remove_leading_path_separator(name);

    let mut state_file_path = [0u8; MAX_PATH_LENGTH];

    // path
    state_file_path[..SHM_STATE_DIRECTORY.len()].copy_from_slice(SHM_STATE_DIRECTORY);

    // name
    let mut name_len = 0;
    for i in 0..usize::MAX {
        let c = *(name.add(i) as *const u8);

        state_file_path[i + SHM_STATE_DIRECTORY.len()] = if c == b'/' { b'\\' } else { c };
        if *(name.add(i)) == 0i8 {
            name_len = i;
            break;
        }
    }

    // suffix
    for i in 0..suffix.len() {
        state_file_path[i + SHM_STATE_DIRECTORY.len() + name_len] = suffix[i];
    }

    state_file_path
}

pub unsafe fn shm_open(name: *const char, oflag: int, mode: mode_t) -> int {
    let ret_val = crate::internal::shm_open(name, oflag, mode as uint);
    if ret_val != -1 {
        let shm_file_path = shm_file_path(name, SHM_STATE_SUFFIX);

        if open_with_mode(shm_file_path.as_ptr().cast(), O_CREAT | O_RDONLY, mode) == -1 {
            close(ret_val);
            return -1;
        }
    }

    ret_val
}

pub unsafe fn shm_unlink(name: *const char) -> int {
    remove(shm_file_path(name, SHM_STATE_SUFFIX).as_ptr().cast());
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

unsafe fn trim_ascii(value: &[i8]) -> &[u8] {
    for i in 0..value.len() {
        if value[i] == 0 {
            return core::slice::from_raw_parts(value.as_ptr().cast(), i);
        }
    }
    core::slice::from_raw_parts(value.as_ptr().cast(), value.len())
}

pub unsafe fn shm_list() -> Vec<[i8; 256]> {
    let mut result = vec![];
    let mut search_path = SHM_STATE_DIRECTORY.to_vec();
    search_path.push(0);
    let dir = opendir(search_path.as_ptr().cast());

    if dir.is_null() {
        return result;
    }

    loop {
        let entry = crate::internal::readdir(dir);
        if entry.is_null() {
            break;
        }

        if (*entry).d_type == crate::internal::DT_REG as _ {
            let file_name = trim_ascii(&(*entry).d_name);
            if file_name.ends_with(SHM_STATE_SUFFIX) {
                let mut shm_name = [0i8; 256];
                for (i, letter) in shm_name
                    .iter_mut()
                    .enumerate()
                    .take(file_name.len() - SHM_STATE_SUFFIX.len())
                {
                    if (*entry).d_name[i] == 0 {
                        break;
                    }

                    *letter = (*entry).d_name[i];
                }

                result.push(shm_name);
            }
        }
    }

    closedir(dir);
    result
}
