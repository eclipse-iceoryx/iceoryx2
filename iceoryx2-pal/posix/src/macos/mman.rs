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

use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU8;
use iceoryx2_pal_configuration::PATH_SEPARATOR;

use core::sync::atomic::Ordering;
use std::io::Write;

use crate::posix::*;

use super::settings::{MAX_PATH_LENGTH, SHM_STATE_DIRECTORY, SHM_STATE_SUFFIX};

const SHM_MAX_NAME_LEN: usize = 33;

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

unsafe fn remove_leading_path_separator(value: *const c_char) -> *const c_char {
    if *value as u8 == PATH_SEPARATOR {
        value.add(1)
    } else {
        value
    }
}

unsafe fn shm_file_path(name: *const c_char, suffix: &[u8]) -> [u8; MAX_PATH_LENGTH] {
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

unsafe fn get_real_shm_name(name: *const c_char) -> Option<[u8; SHM_MAX_NAME_LEN]> {
    let shm_file_path = shm_file_path(name, SHM_STATE_SUFFIX);
    let shm_state_fd = open_with_mode(shm_file_path.as_ptr().cast(), O_RDONLY, 0);
    if shm_state_fd == -1 {
        return None;
    }

    let mut buffer = [0u8; SHM_MAX_NAME_LEN];

    if read(
        shm_state_fd,
        buffer.as_mut_ptr().cast(),
        SHM_MAX_NAME_LEN - 1,
    ) <= 0
    {
        close(shm_state_fd);
        return None;
    }

    close(shm_state_fd);
    Some(buffer)
}

unsafe fn write_real_shm_name(name: *const c_char, buffer: &[u8]) -> bool {
    let shm_file_path = shm_file_path(name, SHM_STATE_SUFFIX);
    let shm_state_fd = open_with_mode(
        shm_file_path.as_ptr().cast(),
        O_EXCL | O_CREAT | O_RDWR,
        S_IWUSR | S_IRUSR | S_IRGRP | S_IROTH,
    );

    if shm_state_fd == -1 {
        return false;
    }

    if write(shm_state_fd, buffer.as_ptr().cast(), buffer.len()) != buffer.len() as _ {
        remove(shm_file_path.as_ptr().cast());
        close(shm_state_fd);
        return false;
    }

    close(shm_state_fd);
    true
}

unsafe fn generate_real_shm_name() -> [u8; SHM_MAX_NAME_LEN] {
    static COUNTER: IoxAtomicU8 = IoxAtomicU8::new(0);

    let mut now = timespec::new_zeroed();
    clock_gettime(CLOCK_REALTIME, &mut now);
    let pid = getpid();
    let shm_name = pid.to_string()
        + "_"
        + &now.tv_sec.to_string()
        + "_"
        // the accuracy is more in the micro second range, therefore dividing by 1000 is safe when
        // we compensate it with the atomic fetch_add
        + &(now.tv_nsec / 1000).to_string()
        + "_"
        + &COUNTER.fetch_add(1, Ordering::Relaxed).to_string();

    let mut buffer = [0u8; SHM_MAX_NAME_LEN];
    buffer.as_mut_slice().write_all(shm_name.as_bytes()).expect(
        "always works since 3 32-bit numbers and 2 underscores do not use more than 33 characters",
    );
    buffer
}

pub unsafe fn shm_open(name: *const c_char, oflag: int, _mode: mode_t) -> int {
    let real_name = get_real_shm_name(name);
    if oflag & O_EXCL != 0 && real_name.is_some() {
        Errno::set(Errno::EEXIST);
        return -1;
    }

    let real_name = match real_name {
        Some(name) => name,
        None => {
            if oflag & O_CREAT == 0 {
                Errno::set(Errno::ENOENT);
                return -1;
            }
            let real_name = generate_real_shm_name();
            if !write_real_shm_name(name, &real_name) {
                return -1;
            }
            real_name
        }
    };

    // TODO iox2-156, shared memory permission cannot be adjusted with fchmod, therefore setting
    //                  it so that the owner can access everything
    let mode = S_IRWXU;
    // TODO iox2-156, end
    crate::internal::shm_open(real_name.as_ptr().cast(), oflag, mode as uint)
}

pub unsafe fn shm_unlink(name: *const c_char) -> int {
    let real_name = get_real_shm_name(name);

    if let Some(real_name) = real_name {
        let ret_val = crate::internal::shm_unlink(real_name.as_ptr().cast());
        if ret_val == 0 || (ret_val == -1 && Errno::get() == Errno::ENOENT) {
            remove(shm_file_path(name, SHM_STATE_SUFFIX).as_ptr().cast());
        }
        return ret_val;
    }

    Errno::set(Errno::ENOENT);
    -1
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
