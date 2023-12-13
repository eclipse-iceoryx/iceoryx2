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

use crate::posix::types::*;

pub unsafe fn scandir(path: *const char, namelist: *mut *mut *mut dirent) -> int {
    internal::scandir_ext(path, namelist)
}

pub unsafe fn mkdir(pathname: *const char, mode: mode_t) -> int {
    crate::internal::mkdir(pathname, mode)
}

pub unsafe fn opendir(dirname: *const char) -> *mut DIR {
    crate::internal::opendir(dirname)
}

pub unsafe fn closedir(dirp: *mut DIR) -> int {
    crate::internal::closedir(dirp)
}

pub unsafe fn dirfd(dirp: *mut DIR) -> int {
    crate::internal::dirfd(dirp)
}

pub unsafe fn readdir(dirp: *mut DIR) -> *const dirent {
    crate::internal::readdir(dirp)
}

mod internal {
    use super::*;

    #[cfg_attr(target_os = "linux", link(name = "c"))]
    extern "C" {
        pub(super) fn scandir_ext(path: *const char, namelist: *mut *mut *mut dirent) -> int;
    }
}
