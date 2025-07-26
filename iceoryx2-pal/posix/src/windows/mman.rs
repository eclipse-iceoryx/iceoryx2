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

use crate::posix::{
    constants::*, settings::*, to_dir_search_string, types::*, Errno, MemZeroedStruct,
};
use crate::win32call;

use iceoryx2_pal_configuration::PATH_SEPARATOR;
use windows_sys::Win32::Foundation::ERROR_FILE_EXISTS;
use windows_sys::Win32::Storage::FileSystem::{
    FindClose, FindFirstFileA, FindNextFileA, FILE_ATTRIBUTE_DIRECTORY, WIN32_FIND_DATAA,
};
use windows_sys::Win32::{
    Foundation::{
        CloseHandle, ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS, ERROR_FILE_NOT_FOUND, FALSE,
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

use super::win32_handle_translator::{FdHandleEntry, FileHandle, HandleTranslator, ShmHandle};

const MAX_SUPPORTED_SHM_SIZE: u64 = 128 * 1024 * 1024 * 1024;

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

unsafe fn remove_leading_path_separator(value: *const c_char) -> *const c_char {
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
    let mut data = WIN32_FIND_DATAA::new_zeroed();
    let (handle, _) = win32call! { FindFirstFileA(search_path.as_ptr().cast(), &mut data), ignore ERROR_FILE_NOT_FOUND };

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

        let (file_found, _) = win32call! { FindNextFileA(handle, &mut data) };
        if file_found == FALSE {
            break;
        }
    }

    win32call! { FindClose(handle) };

    result
}

pub unsafe fn shm_open(name: *const c_char, oflag: int, mode: mode_t) -> int {
    let name = remove_leading_path_separator(name.cast());
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

        let last_mapping_error;
        (shm_handle, last_mapping_error) = win32call! {CreateFileMappingA(
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

        if oflag & O_EXCL != 0 && last_mapping_error == ERROR_ALREADY_EXISTS {
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

        let last_mapping_error;
        (shm_handle, last_mapping_error) = win32call! {OpenFileMappingA(FILE_MAP_ALL_ACCESS, false as i32, name as *const u8), ignore ERROR_FILE_NOT_FOUND};

        if shm_handle == 0 {
            Errno::set(Errno::ENOENT);
            shm_unlink(name);
            win32call! {CloseHandle(shm_state_handle)};
            return -1;
        }

        if last_mapping_error != 0 {
            Errno::set(Errno::EACCES);
            win32call! {CloseHandle(shm_handle)};
            win32call! {CloseHandle(shm_state_handle)};
            return -1;
        }
    }

    HandleTranslator::get_instance().add(FdHandleEntry::SharedMemory(ShmHandle {
        handle: FileHandle {
            handle: shm_handle,
            lock_state: F_UNLCK,
        },
        state_handle: shm_state_handle,
    }))
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

unsafe fn create_state_handle(name: *const c_char) -> HANDLE {
    let name = remove_leading_path_separator(name);

    let create_file = || {
        let (handle, last_error) = win32call! {CreateFileA(
            shm_file_path(name, SHM_STATE_SUFFIX).as_ptr(),
            GENERIC_WRITE | GENERIC_READ,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            core::ptr::null::<SECURITY_ATTRIBUTES>(),
            CREATE_NEW,
            FILE_ATTRIBUTE_NORMAL,
            0,
        ), ignore ERROR_FILE_EXISTS};
        (handle, last_error)
    };

    let (mut handle, last_error) = create_file();
    if handle == INVALID_HANDLE_VALUE && last_error == ERROR_FILE_EXISTS && !does_shm_exist(name) {
        remove_state_handle(name);
        (handle, _) = create_file();
    }

    handle
}

unsafe fn open_state_handle(name: *const c_char) -> HANDLE {
    let name = remove_leading_path_separator(name);

    let (handle, _) = win32call! {CreateFileA(
        shm_file_path(name, SHM_STATE_SUFFIX).as_ptr(),
        GENERIC_READ,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        core::ptr::null::<SECURITY_ATTRIBUTES>(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        0,
    ), ignore ERROR_FILE_NOT_FOUND };
    handle
}

unsafe fn remove_state_handle(name: *const c_char) -> int {
    let name = remove_leading_path_separator(name);

    let (has_deleted_file, error_code) = win32call! { DeleteFileA(shm_file_path(name, SHM_STATE_SUFFIX).as_ptr()),
    ignore ERROR_FILE_NOT_FOUND, ERROR_ACCESS_DENIED};
    if has_deleted_file == FALSE {
        // TODO: [#9]
        Errno::set(Errno::ENOENT);
        return -1;
    }
    0
}

unsafe fn does_shm_exist(name: *const c_char) -> bool {
    let (shm_handle, last_error) = win32call! {OpenFileMappingA(FILE_MAP_ALL_ACCESS, false as i32, name as *const u8), ignore ERROR_FILE_NOT_FOUND};
    !(shm_handle == 0 && last_error == ERROR_FILE_NOT_FOUND)
}

pub(crate) unsafe fn shm_set_size(fd_handle: HANDLE, shm_size: u64) {
    if fd_handle == INVALID_HANDLE_VALUE {
        return;
    }

    if shm_size > MAX_SUPPORTED_SHM_SIZE {
        eprintln!("Trying to allocate {shm_size} which is larger than the maximum supported shared memory size of {MAX_SUPPORTED_SHM_SIZE}");
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
    let (has_read_file, _) = win32call! { ReadFile(
        fd_handle,
        (&mut read_buffer as *mut u64) as *mut void,
        8,
        &mut bytes_read,
        core::ptr::null_mut::<OVERLAPPED>(),
    )};
    if has_read_file == FALSE || bytes_read != 8 {
        read_buffer = 0;
    }

    read_buffer
}

pub unsafe fn shm_unlink(name: *const c_char) -> int {
    remove_state_handle(name)
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
        Some(FdHandleEntry::SharedMemory(v)) => v,
        _ => {
            Errno::set(Errno::EINVAL);
            return core::ptr::null_mut::<void>();
        }
    };

    let (map_result, _) =
        win32call! { MapViewOfFile(win_handle.handle.handle, FILE_MAP_ALL_ACCESS, 0, 0, len)};
    match map_result {
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
    let (has_unmapped, _) = win32call! { UnmapViewOfFile(addr as _) };
    if has_unmapped == FALSE {
        Errno::set(Errno::EINVAL);
        return -1;
    }
    0
}

pub unsafe fn mprotect(addr: *mut void, len: size_t, prot: int) -> int {
    -1
}
