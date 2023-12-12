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

use core::sync::atomic::{AtomicU64, Ordering};
use windows_sys::Win32::{
    Foundation::{
        ERROR_ALREADY_EXISTS, ERROR_FILE_NOT_FOUND, ERROR_NO_MORE_FILES, ERROR_PATH_NOT_FOUND,
        INVALID_HANDLE_VALUE,
    },
    Security::SECURITY_ATTRIBUTES,
    Storage::FileSystem::{
        CreateDirectoryA, FindClose, FindFirstFileA, FindNextFileA, WIN32_FIND_DATAA,
    },
};

use crate::{
    posix::Struct,
    posix::{self},
    posix::{
        types::*,
        win32_handle_translator::{FdHandleEntry, HandleTranslator},
    },
    win32call,
    windows::win32_udp_port_to_uds_name::MAX_UDS_NAME_LEN,
};

use super::settings::MAX_PATH_LENGTH;

impl Struct for WIN32_FIND_DATAA {}

unsafe fn number_of_directory_entries(path: &[u8]) -> usize {
    let mut data = WIN32_FIND_DATAA::new();
    let handle =
        win32call! { FindFirstFileA(path.as_ptr(), &mut data), ignore ERROR_FILE_NOT_FOUND};

    if handle == INVALID_HANDLE_VALUE {
        return 0;
    }

    let mut number_of_entries = 0;
    loop {
        if win32call! {FindNextFileA(handle, &mut data), ignore ERROR_NO_MORE_FILES} == 0 {
            break;
        }
        number_of_entries += 1;
    }

    win32call! {FindClose(handle)};
    number_of_entries
}

pub(crate) unsafe fn to_dir_search_string(path: *const char) -> [u8; MAX_PATH_LENGTH] {
    let mut buffer = [0u8; MAX_PATH_LENGTH];

    for i in 0..MAX_PATH_LENGTH {
        let c = *path.add(i) as u8;
        if c == b'\0' {
            buffer[i] = b'\\';
            buffer[i + 1] = b'*';
            break;
        }
        buffer[i] = *path.add(i) as u8;
    }

    buffer
}

pub unsafe fn scandir(path: *const char, namelist: *mut *mut *mut dirent) -> int {
    let uds_files = HandleTranslator::get_instance().list_all_uds(path);
    let path = to_dir_search_string(path);
    let number_of_entries = number_of_directory_entries(&path) + uds_files.len();
    let mut data = WIN32_FIND_DATAA::new();
    let handle =
        win32call! { FindFirstFileA(path.as_ptr(), &mut data), ignore ERROR_FILE_NOT_FOUND};

    if handle == INVALID_HANDLE_VALUE {
        return -1;
    }

    *namelist =
        posix::malloc(core::mem::size_of::<*mut dirent>() * number_of_entries) as *mut *mut dirent;

    let mut i = 0;
    for file in &uds_files {
        let entry_ptr: *mut dirent = posix::malloc(core::mem::size_of::<dirent>()) as *mut dirent;
        let entry = &mut *entry_ptr;

        entry.d_name[..file.len()].copy_from_slice(core::mem::transmute::<
            &[u8; MAX_UDS_NAME_LEN],
            &[i8; MAX_UDS_NAME_LEN],
        >(file));
        *(*namelist).add(i) = entry_ptr;

        i += 1;
    }

    loop {
        if win32call! {FindNextFileA(handle, &mut data), ignore ERROR_NO_MORE_FILES} == 0
            || i >= number_of_entries
        {
            break;
        }

        let entry_ptr: *mut dirent = posix::malloc(core::mem::size_of::<dirent>()) as *mut dirent;
        let entry = &mut *entry_ptr;

        entry.d_name = core::array::from_fn(|i| data.cFileName[i] as i8);

        *(*namelist).add(i) = entry_ptr;
        i += 1;
    }

    win32call! {FindClose(handle)};
    number_of_entries as int
}

pub unsafe fn mkdir(pathname: *const char, mode: mode_t) -> int {
    if win32call! { CreateDirectoryA(pathname as *const u8, core::ptr::null::<SECURITY_ATTRIBUTES>()),
    ignore ERROR_ALREADY_EXISTS, ERROR_PATH_NOT_FOUND}
        == 0
    {
        return -1;
    }
    0
}

pub unsafe fn opendir(dirname: *const char) -> *mut DIR {
    static COUNT: AtomicU64 = AtomicU64::new(1);
    let id = COUNT.fetch_add(1, Ordering::Relaxed);

    HandleTranslator::get_instance().add(FdHandleEntry::DirectoryStream(id));
    id as *mut DIR
}

pub unsafe fn closedir(dirp: *mut DIR) -> int {
    HandleTranslator::get_instance().remove_entry(FdHandleEntry::DirectoryStream(dirp as u64));
    0
}

pub unsafe fn dirfd(dirp: *mut DIR) -> int {
    HandleTranslator::get_instance().get_fd(FdHandleEntry::DirectoryStream(dirp as u64))
}
