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

#![allow(non_camel_case_types)]

use iceoryx2::prelude::*;

use core::ffi::c_void;

// BEGIN type definition

pub type iox2_config_h = *const c_void;

// END type definition

// BEGIN C API
#[no_mangle]
pub extern "C" fn iox2_config_global_config() -> iox2_config_h {
    Config::global_config() as *const _ as *const _
}

// END C API
