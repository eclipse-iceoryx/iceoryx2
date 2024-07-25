// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use crate::posix::types::*;
use crate::posix::{closedir, free, malloc, opendir, readdir_r};

pub unsafe fn scandir_impl(
    path: *const c_char,
    namelist: *mut *mut *mut crate::posix::types::dirent,
) -> int {
    let dirfd = opendir(path);
    if dirfd.is_null() {
        return -1;
    }

    *namelist = core::ptr::null_mut::<*mut dirent>();
    let mut entries = vec![];
    const DIRENT_SIZE: usize = core::mem::size_of::<dirent>();

    let cleanup = |entries: &Vec<*mut void>, namelist: *mut *mut dirent| {
        for entry in entries {
            free((*entry).cast());
        }

        if !namelist.is_null() {
            free(namelist.cast());
        }
    };

    loop {
        let dirent_ptr = malloc(DIRENT_SIZE);
        let result_ptr: *mut *mut dirent = malloc(core::mem::size_of::<*mut dirent>()).cast();

        if readdir_r(dirfd, dirent_ptr.cast(), result_ptr) != 0 {
            free(result_ptr.cast());
            free(dirent_ptr);
            cleanup(&entries, *namelist);

            closedir(dirfd);
            return -1;
        }

        if (*result_ptr).is_null() {
            free(result_ptr.cast());
            free(dirent_ptr);
            break;
        }

        free(result_ptr.cast());
        entries.push(dirent_ptr);
    }

    *namelist = malloc(core::mem::size_of::<*mut *mut dirent>() * entries.len()).cast();
    if (*namelist).is_null() {
        cleanup(&entries, *namelist);
        closedir(dirfd);
        return -1;
    }

    for (n, entry) in entries.iter().enumerate() {
        (*namelist).add(n).write((*entry).cast());
    }

    closedir(dirfd);
    entries.len() as _
}
