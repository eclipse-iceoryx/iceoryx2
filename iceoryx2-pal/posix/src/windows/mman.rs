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
#![allow(unused_variables)]

use crate::posix::{constants::*, settings::*, to_dir_search_string, types::*, Errno, Struct};
use crate::win32call;

use iceoryx2_pal_settings::PATH_SEPARATOR;
use windows_sys::Win32::Storage::FileSystem::{
    FindClose, FindFirstFileA, FindNextFileA, FILE_ATTRIBUTE_DIRECTORY, WIN32_FIND_DATAA,
};
use windows_sys::Win32::{
    Foundation::{
        CloseHandle, GetLastError, ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS, ERROR_FILE_NOT_FOUND,
        GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE,
    },
    Security::SECURITY_ATTRIBUTES,
    Storage::FileSystem::{
        CreateFileA, DeleteFileA, ReadFile, SetFilePointer, WriteFile, CREATE_NEW,
        FILE_ATTRIBUTE_NORMAL, FILE_BEGIN, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
        OPEN_EXISTING,
    },
    System::{Memory::*, IO::OVERLAPPED},
};

use super::win32_handle_translator::{FdHandleEntry, HandleTranslator, Win32Handle};

const MAX_SUPPORTED_SHM_SIZE: usize = 1024 * 1024 * 1024;

pub unsafe fn mlock(addr: *const void, len: size_t) -> int {
    -1
}

pub unsafe fn munlock(addr: *const void, len: size_t) -> int {
    -1
}

pub unsafe fn mlockall(flags: int) -> int {
    -1
}

pub unsafe fn munlockall() -> int {
    -1
}

unsafe fn remove_leading_path_separator(value: *const char) -> *const char {
    if *value as u8 == PATH_SEPARATOR {
        value.offset(1)
    } else {
        value
    }
}

unsafe fn trim_ascii(value: &[u8]) -> &[u8] {
    for i in 0..value.len() {
        if value[i] == 0 {
            return value.split_at(i).0;
        }
    }

    value
}

pub unsafe fn shm_list() -> Vec<[i8; 256]> {
    let mut result = vec![];
    let mut search_path = SHM_STATE_DIRECTORY.to_vec();
    search_path.push(0);
    let search_path = to_dir_search_string(search_path.as_ptr().cast());

    //SHM_STATE_SUFFIX
    let mut data = WIN32_FIND_DATAA::new();
    let handle = win32call! { FindFirstFileA(search_path.as_ptr().cast(), &mut data), ignore ERROR_FILE_NOT_FOUND };

    if handle == INVALID_HANDLE_VALUE {
        return result;
    }

    loop {
        if data.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0 {
            let file_name = trim_ascii(&data.cFileName);
            if file_name.ends_with(SHM_STATE_SUFFIX) {
                let name = file_name
                    .split_at(file_name.len() - SHM_STATE_SUFFIX.len())
                    .0;

                let mut shm_name = [0i8; 256];
                for i in 0..core::cmp::min(shm_name.len(), name.len()) {
                    shm_name[i] = name[i] as _;
                }
                result.push(shm_name);
            }
        }

        if win32call! { FindNextFileA(handle, &mut data) } == 0 {
            break;
        }
    }

    win32call! { FindClose(handle) };

    result
}

pub unsafe fn shm_open(name: *const char, oflag: int, mode: mode_t) -> int {
    let name = remove_leading_path_separator(name);
    let handle: HANDLE = 0;
    let shm_handle;
    let mut shm_state_handle;

    if oflag & O_CREAT != 0 {
        shm_state_handle = create_state_handle(name);
        if shm_state_handle == INVALID_HANDLE_VALUE {
            if oflag & O_EXCL != 0 {
                Errno::set(Errno::EEXIST);
                return -1;
            }

            shm_state_handle = open_state_handle(name);

            if shm_state_handle == INVALID_HANDLE_VALUE {
                Errno::set(Errno::ENOENT);
                return -1;
            }
        }
        shm_set_size(shm_state_handle, 0);

        const MAX_SIZE_LOW: u32 = (MAX_SUPPORTED_SHM_SIZE & 0xFFFFFFFF) as u32;
        const MAX_SIZE_HIGH: u32 = ((MAX_SUPPORTED_SHM_SIZE >> 32) & 0xFFFFFFFF) as u32;

        shm_handle = win32call! {CreateFileMappingA(
            handle,
            core::ptr::null::<SECURITY_ATTRIBUTES>(),
            PAGE_READWRITE | SEC_RESERVE,
            MAX_SIZE_HIGH,
            MAX_SIZE_LOW,
            name as *const u8,
        ), ignore ERROR_ALREADY_EXISTS};

        if shm_handle == 0 {
            Errno::set(Errno::EACCES);
            CloseHandle(shm_state_handle);
            return -1;
        }

        if oflag & O_EXCL != 0 && GetLastError() == ERROR_ALREADY_EXISTS {
            CloseHandle(shm_handle);
            CloseHandle(shm_state_handle);
            return -1;
        }
    } else {
        shm_state_handle = open_state_handle(name);

        if shm_state_handle == INVALID_HANDLE_VALUE {
            Errno::set(Errno::ENOENT);
            return -1;
        }

        shm_handle =
            win32call! {OpenFileMappingA(FILE_MAP_ALL_ACCESS, false as i32, name as *const u8)};

        if shm_handle == 0 {
            Errno::set(Errno::ENOENT);
            win32call! {CloseHandle(shm_state_handle)};
            return -1;
        }

        if GetLastError() != 0 {
            Errno::set(Errno::EACCES);
            win32call! {CloseHandle(shm_handle)};
            win32call! {CloseHandle(shm_state_handle)};
            return -1;
        }
    }

    HandleTranslator::get_instance().add(FdHandleEntry::Handle(Win32Handle {
        handle: shm_handle,
        state_handle: shm_state_handle,
        lock_state: F_UNLCK,
    }))
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

unsafe fn create_state_handle(name: *const char) -> HANDLE {
    let name = remove_leading_path_separator(name);

    win32call! {CreateFileA(
        shm_file_path(name, SHM_STATE_SUFFIX).as_ptr(),
        GENERIC_WRITE | GENERIC_READ,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        core::ptr::null::<SECURITY_ATTRIBUTES>(),
        CREATE_NEW,
        FILE_ATTRIBUTE_NORMAL,
        0,
    )}
}

unsafe fn open_state_handle(name: *const char) -> HANDLE {
    let name = remove_leading_path_separator(name);

    win32call! {CreateFileA(
        shm_file_path(name, SHM_STATE_SUFFIX).as_ptr(),
        GENERIC_READ,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        core::ptr::null::<SECURITY_ATTRIBUTES>(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        0,
    ), ignore ERROR_FILE_NOT_FOUND }
}

pub(crate) unsafe fn shm_set_size(fd_handle: HANDLE, shm_size: u64) {
    if fd_handle == INVALID_HANDLE_VALUE {
        return;
    }

    let mut bytes_written = 0;

    win32call! {SetFilePointer(fd_handle, 0, core::ptr::null_mut::<i32>(), FILE_BEGIN)};
    win32call! { WriteFile(
        fd_handle,
        (&shm_size as *const u64) as *const u8,
        8,
        &mut bytes_written,
        core::ptr::null_mut::<OVERLAPPED>(),
    )};
}

pub(crate) unsafe fn shm_get_size(fd_handle: HANDLE) -> u64 {
    if fd_handle == INVALID_HANDLE_VALUE {
        return 0;
    }

    let mut read_buffer: u64 = 0;
    let mut bytes_read = 0;

    win32call! {SetFilePointer(fd_handle, 0, core::ptr::null_mut::<i32>(), FILE_BEGIN)};
    if win32call! { ReadFile(
        fd_handle,
        (&mut read_buffer as *mut u64) as *mut void,
        8,
        &mut bytes_read,
        core::ptr::null_mut::<OVERLAPPED>(),
    )} == 0
        || bytes_read != 8
    {
        read_buffer = 0;
    }

    read_buffer
}

pub unsafe fn shm_unlink(name: *const char) -> int {
    let name = remove_leading_path_separator(name);

    if win32call! { DeleteFileA(shm_file_path(name, SHM_STATE_SUFFIX).as_ptr()),
    ignore ERROR_FILE_NOT_FOUND, ERROR_ACCESS_DENIED}
        == 0
    {
        // TODO: [#41]
        Errno::set(Errno::ENOENT);
        return -1;
    }
    0
}

pub unsafe fn mmap(
    addr: *mut void,
    len: size_t,
    prot: int,
    flags: int,
    fd: int,
    off: off_t,
) -> *mut void {
    if len == 0 {
        Errno::set(Errno::EINVAL);
        return core::ptr::null_mut::<void>();
    }

    let win_handle = match HandleTranslator::get_instance().get(fd) {
        Some(FdHandleEntry::Handle(v)) => v,
        _ => {
            Errno::set(Errno::EINVAL);
            return core::ptr::null_mut::<void>();
        }
    };

    match win32call! { MapViewOfFile(win_handle.handle, FILE_MAP_ALL_ACCESS, 0, 0, len)} {
        0 => {
            Errno::set(Errno::ENOMEM);
            core::ptr::null_mut::<void>()
        }
        lpaddress => {
            if VirtualAlloc(lpaddress as *const void, len, MEM_COMMIT, PAGE_READWRITE).is_null() {
                Errno::set(Errno::ENOMEM);
                return core::ptr::null_mut::<void>();
            }
            lpaddress as *mut void
        }
    }
}

pub unsafe fn munmap(addr: *mut void, len: size_t) -> int {
    if win32call! { UnmapViewOfFile(addr as _) } == 0 {
        Errno::set(Errno::EINVAL);
        return -1;
    }
    0
}

pub unsafe fn mprotect(addr: *mut void, len: size_t, prot: int) -> int {
    -1
}
