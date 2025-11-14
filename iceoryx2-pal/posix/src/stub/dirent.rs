// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use core::unimplemented;

use crate::posix::types::*;

pub unsafe fn scandir(path: *const c_char, namelist: *mut *mut *mut dirent) -> int {
    unimplemented!("scandir")
}

pub unsafe fn mkdir(pathname: *const c_char, mode: mode_t) -> int {
    unimplemented!("mkdir")
}

pub unsafe fn opendir(dirname: *const c_char) -> *mut DIR {
    unimplemented!("opendir")
}

pub unsafe fn closedir(dirp: *mut DIR) -> int {
    unimplemented!("closedir")
}

pub unsafe fn dirfd(dirp: *mut DIR) -> int {
    unimplemented!("dirfd")
}

pub unsafe fn readdir_r(dirp: *mut DIR, entry: *mut dirent, result: *mut *mut dirent) -> int {
    unimplemented!("readdir_r")
}

pub fn dirent_size() -> usize {
    core::mem::size_of::<crate::posix::types::dirent>()
}
