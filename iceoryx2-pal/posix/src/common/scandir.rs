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

use crate::posix::{closedir, free, malloc, opendir, readdir_r};
use crate::posix::{dirent_size, types::*};

pub(crate) unsafe fn scandir_impl(
    path: *const c_char,
    namelist: *mut *mut *mut crate::posix::types::dirent,
) -> int {
    let dirfd = opendir(path);
    if dirfd.is_null() {
        return -1;
    }

    *namelist = core::ptr::null_mut::<*mut dirent>();
    let mut entries = vec![];

    let cleanup = |entries: &mut Vec<*mut void>, namelist: *mut *mut *mut dirent| {
        entries.drain(..).for_each(|entry| {
            free(entry);
        });

        if !(*namelist).is_null() {
            free((*namelist).cast());
        }
        *namelist = core::ptr::null_mut();
    };

    loop {
        let dirent_ptr = malloc(dirent_size());

        let mut result_ptr: *mut dirent = core::ptr::null_mut();

        if readdir_r(dirfd, dirent_ptr.cast(), &mut result_ptr as _) != 0 {
            free(dirent_ptr);
            cleanup(&mut entries, namelist);

            closedir(dirfd);
            return -1;
        }

        if result_ptr.is_null() {
            free(dirent_ptr);
            break;
        }

        entries.push(dirent_ptr);
    }

    let num_entries = entries.len();
    *namelist = malloc(core::mem::size_of::<*mut *mut dirent>() * num_entries).cast();
    if (*namelist).is_null() {
        cleanup(&mut entries, namelist);
        closedir(dirfd);
        return -1;
    }

    entries.drain(..).enumerate().for_each(|(n, entry)| {
        (*namelist).add(n).write(entry.cast());
    });

    closedir(dirfd);
    num_entries as _
}
