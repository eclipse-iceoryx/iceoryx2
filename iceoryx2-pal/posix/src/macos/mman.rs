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

use iceoryx2_pal_concurrency_sync::atomic::AtomicU8;
use iceoryx2_pal_configuration::PATH_SEPARATOR;

use iceoryx2_pal_concurrency_sync::atomic::Ordering;
use std::io::Write;

use crate::posix::*;

use super::macos_fd_translator::ShmFdTranslator;
use super::settings::{MAX_PATH_LENGTH, SHM_STATE_DIRECTORY, SHM_STATE_SUFFIX};

const SHM_MAX_NAME_LEN: usize = 33;

// iox2-156: user mode bytes live at this offset in the .shm_state file.
pub(crate) const SHM_STATE_MODE_OFFSET: i64 = SHM_MAX_NAME_LEN as i64;
pub(crate) const SHM_STATE_MODE_LEN: usize = 4;

pub unsafe fn mlock(addr: *const void, len: size_t) -> int {
    unsafe { crate::internal::mlock(addr, len) }
}

pub unsafe fn munlock(addr: *const void, len: size_t) -> int {
    unsafe { crate::internal::munlock(addr, len) }
}

pub unsafe fn mlockall(flags: int) -> int {
    unsafe { crate::internal::mlockall(flags) }
}

pub unsafe fn munlockall() -> int {
    unsafe { crate::internal::munlockall() }
}

unsafe fn remove_leading_path_separator(value: *const c_char) -> *const c_char {
    unsafe {
        if *value as u8 == PATH_SEPARATOR {
            value.add(1)
        } else {
            value
        }
    }
}

unsafe fn shm_file_path(name: *const c_char, suffix: &[u8]) -> [u8; MAX_PATH_LENGTH] {
    let name = unsafe { remove_leading_path_separator(name) };

    let mut state_file_path = [0u8; MAX_PATH_LENGTH];

    // path
    state_file_path[..SHM_STATE_DIRECTORY.len()].copy_from_slice(SHM_STATE_DIRECTORY);

    // name
    let mut name_len = 0;
    for i in 0..usize::MAX {
        let c = unsafe { *(name.add(i) as *const u8) };

        state_file_path[i + SHM_STATE_DIRECTORY.len()] = if c == b'/' { b'\\' } else { c };
        unsafe {
            if *(name.add(i)) == 0i8 {
                name_len = i;
                break;
            }
        }
    }

    // suffix
    for i in 0..suffix.len() {
        state_file_path[i + SHM_STATE_DIRECTORY.len() + name_len] = suffix[i];
    }

    state_file_path
}

unsafe fn open_state_file(name: *const c_char) -> Option<([u8; SHM_MAX_NAME_LEN], int)> {
    unsafe {
        let shm_file_path = shm_file_path(name, SHM_STATE_SUFFIX);
        // O_RDWR so iox2-156 routing can pwrite mode bytes later.
        let state_fd = open_with_mode(shm_file_path.as_ptr().cast(), O_RDWR, 0);
        if state_fd == -1 {
            return None;
        }

        let mut buffer = [0u8; SHM_MAX_NAME_LEN];
        if read(state_fd, buffer.as_mut_ptr().cast(), SHM_MAX_NAME_LEN - 1) <= 0 {
            close(state_fd);
            return None;
        }

        Some((buffer, state_fd))
    }
}

unsafe fn create_state_file(name: *const c_char, real_name: &[u8], mode: mode_t) -> Option<int> {
    unsafe {
        let shm_file_path = shm_file_path(name, SHM_STATE_SUFFIX);
        let state_fd = open_with_mode(
            shm_file_path.as_ptr().cast(),
            O_EXCL | O_CREAT | O_RDWR,
            S_IWUSR | S_IRUSR,
        );
        if state_fd == -1 {
            return None;
        }

        if write(state_fd, real_name.as_ptr().cast(), real_name.len()) != real_name.len() as _ {
            close(state_fd);
            remove(shm_file_path.as_ptr().cast());
            return None;
        }

        if !write_state_mode(state_fd, mode) {
            close(state_fd);
            remove(shm_file_path.as_ptr().cast());
            return None;
        }

        Some(state_fd)
    }
}

pub(crate) unsafe fn write_state_mode(state_fd: int, mode: mode_t) -> bool {
    let bytes = (mode as u32).to_le_bytes();
    let written = unsafe {
        crate::internal::pwrite(
            state_fd,
            bytes.as_ptr().cast(),
            SHM_STATE_MODE_LEN,
            SHM_STATE_MODE_OFFSET,
        )
    };
    written == SHM_STATE_MODE_LEN as _
}

pub(crate) unsafe fn read_state_mode(state_fd: int) -> mode_t {
    let mut bytes = [0u8; SHM_STATE_MODE_LEN];
    let n = unsafe {
        crate::internal::pread(
            state_fd,
            bytes.as_mut_ptr().cast(),
            SHM_STATE_MODE_LEN,
            SHM_STATE_MODE_OFFSET,
        )
    };
    if n != SHM_STATE_MODE_LEN as _ {
        return 0 as mode_t;
    }
    u32::from_le_bytes(bytes) as mode_t
}

unsafe fn generate_real_shm_name() -> [u8; SHM_MAX_NAME_LEN] {
    static COUNTER: AtomicU8 = AtomicU8::new(0);

    let mut now = timespec::new_zeroed();
    unsafe { clock_gettime(CLOCK_REALTIME, &mut now) };
    let pid = unsafe { getpid() };
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

pub unsafe fn shm_open(name: *const c_char, oflag: int, mode: mode_t) -> int {
    let existing = unsafe { open_state_file(name) };
    if oflag & O_EXCL != 0 && existing.is_some() {
        if let Some((_, state_fd)) = existing {
            unsafe { close(state_fd) };
        }
        Errno::set(Errno::EEXIST);
        return -1;
    }

    let (real_name, state_fd, created) = match existing {
        Some((real_name, state_fd)) => (real_name, state_fd, false),
        None => {
            if oflag & O_CREAT == 0 {
                Errno::set(Errno::ENOENT);
                return -1;
            }
            let real_name = unsafe { generate_real_shm_name() };
            let state_fd = match unsafe { create_state_file(name, &real_name, mode) } {
                Some(fd) => fd,
                None => return -1,
            };
            (real_name, state_fd, true)
        }
    };

    // macOS ignores shm_open mode; iox2-156 user mode lives on the trampoline.
    let shm_fd =
        unsafe { crate::internal::shm_open(real_name.as_ptr().cast(), oflag, S_IRWXU as uint) };
    if shm_fd == -1 {
        let err = Errno::get();
        unsafe { close(state_fd) };
        if created {
            unsafe { remove(shm_file_path(name, SHM_STATE_SUFFIX).as_ptr().cast()) };
        }
        Errno::set(err);
        return -1;
    }

    if !ShmFdTranslator::get_instance().register(shm_fd, state_fd) {
        unsafe {
            crate::internal::close(shm_fd);
            close(state_fd);
        }
        if created {
            unsafe { remove(shm_file_path(name, SHM_STATE_SUFFIX).as_ptr().cast()) };
        }
        Errno::set(Errno::ENFILE);
        return -1;
    }

    shm_fd
}

pub unsafe fn shm_unlink(name: *const c_char) -> int {
    let real_name = match unsafe { open_state_file(name) } {
        Some((real_name, state_fd)) => {
            unsafe { close(state_fd) };
            real_name
        }
        None => {
            Errno::set(Errno::ENOENT);
            return -1;
        }
    };

    let ret_val = unsafe { crate::internal::shm_unlink(real_name.as_ptr().cast()) };
    if ret_val == 0 || (ret_val == -1 && Errno::get() == Errno::ENOENT) {
        unsafe { remove(shm_file_path(name, SHM_STATE_SUFFIX).as_ptr().cast()) };
    }
    ret_val
}

pub unsafe fn mmap(
    addr: *mut void,
    len: size_t,
    prot: int,
    flags: int,
    fd: int,
    off: off_t,
) -> *mut void {
    unsafe { crate::internal::mmap(addr, len, prot, flags, fd, off) }
}

pub unsafe fn munmap(addr: *mut void, len: size_t) -> int {
    unsafe { crate::internal::munmap(addr, len) }
}

pub unsafe fn mprotect(addr: *mut void, len: size_t, prot: int) -> int {
    unsafe { crate::internal::mprotect(addr, len, prot) }
}

unsafe fn trim_ascii(value: &[i8]) -> &[u8] {
    for i in 0..value.len() {
        if value[i] == 0 {
            return unsafe { core::slice::from_raw_parts(value.as_ptr().cast(), i) };
        }
    }
    unsafe { core::slice::from_raw_parts(value.as_ptr().cast(), value.len()) }
}

pub unsafe fn shm_list() -> Vec<[i8; 256]> {
    let mut result = vec![];
    let mut search_path = SHM_STATE_DIRECTORY.to_vec();
    search_path.push(0);
    unsafe {
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
}
