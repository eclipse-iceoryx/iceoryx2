// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_elementary_traits::AsCStr;
use iceoryx2_ffi_macros::CStrRepr;

use crate::IOX2_OK;

#[repr(C)]
#[derive(Copy, Clone, CStrRepr)]
pub enum iox2_flatbuffer_find_schema_file_error_e {
    INVALID_TYPE_NAME_CHARACTERS = IOX2_OK as isize + 1,
    INVALID_TYPE_NAMESPACE_CHARACTERS,
    INVALID_ROOT_PATH,
    BUFFER_TOO_SMALL,
    NO_SCHEMA_FILE_FOUND,
}
