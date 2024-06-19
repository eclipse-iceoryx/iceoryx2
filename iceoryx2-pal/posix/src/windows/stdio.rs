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

use windows_sys::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_FILE_NOT_FOUND, FALSE};
use windows_sys::Win32::Storage::FileSystem::DeleteFileA;

use crate::posix::types::*;

use crate::win32call;

use super::win32_handle_translator::HandleTranslator;

pub unsafe fn remove(pathname: *const c_char) -> int {
    let (has_deleted, _) = win32call! { DeleteFileA(pathname as *const u8), ignore ERROR_FILE_NOT_FOUND, ERROR_ACCESS_DENIED };
    if has_deleted == FALSE {
        if HandleTranslator::get_instance().remove_uds(pathname) {
            return 0;
        }

        return -1;
    }

    0
}
