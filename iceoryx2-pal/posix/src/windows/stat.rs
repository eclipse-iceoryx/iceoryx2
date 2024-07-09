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

#![allow(non_camel_case_types, non_snake_case)]
#![allow(clippy::missing_safety_doc)]
#![allow(unused_variables)]

use windows_sys::Win32::{
    Foundation::ERROR_FILE_NOT_FOUND,
    Storage::FileSystem::{GetFileAttributesA, FILE_ATTRIBUTE_DIRECTORY, INVALID_FILE_ATTRIBUTES},
};

use crate::posix::*;

use crate::win32call;

use super::win32_handle_translator::HandleTranslator;

pub unsafe fn stat(path: *const c_char, buf: *mut stat_t) -> int {
    if HandleTranslator::get_instance().contains_uds(path.cast()) {
        (*buf).st_mode = S_IFSOCK | S_IRUSR | S_IWUSR | S_IXUSR;
        return 0;
    }

    let (attr, _) =
        win32call! { GetFileAttributesA(path as *const u8), ignore ERROR_FILE_NOT_FOUND};
    if attr == INVALID_FILE_ATTRIBUTES {
        Errno::set(Errno::ENOENT);
        return -1;
    }

    if attr & FILE_ATTRIBUTE_DIRECTORY != 0 {
        (*buf).st_mode = S_IFDIR;
    } else {
        (*buf).st_mode = S_IFREG;
    }

    if let Some(mode) = acquire_mode_from_path(core::slice::from_raw_parts(
        path as *const u8,
        c_string_length(path),
    )) {
        (*buf).st_mode |= mode;
    } else {
        Errno::set(Errno::ENOENT);
        return -1;
    }

    0
}

pub unsafe fn umask(mask: mode_t) -> mode_t {
    mode_t::MAX
}
