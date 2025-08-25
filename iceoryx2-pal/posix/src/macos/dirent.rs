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

use crate::{common::scandir::scandir_impl, posix::types::*};

pub unsafe fn scandir(path: *const c_char, namelist: *mut *mut *mut dirent) -> int {
    scandir_impl(path, namelist)
}

pub unsafe fn mkdir(pathname: *const c_char, mode: mode_t) -> int {
    crate::internal::mkdir(pathname, mode)
}

pub unsafe fn opendir(dirname: *const c_char) -> *mut DIR {
    crate::internal::opendir(dirname)
}

pub unsafe fn closedir(dirp: *mut DIR) -> int {
    crate::internal::closedir(dirp)
}

pub unsafe fn dirfd(dirp: *mut DIR) -> int {
    internal::dirfd(dirp)
}

pub unsafe fn readdir_r(dirp: *mut DIR, entry: *mut dirent, result: *mut *mut dirent) -> int {
    crate::internal::readdir_r(dirp, entry, result)
}

pub fn dirent_size() -> usize {
    core::mem::size_of::<crate::posix::types::dirent>()
}

mod internal {
    use super::*;

    extern "C" {
        pub(super) fn dirfd(dirp: *mut DIR) -> int;
    }
}
