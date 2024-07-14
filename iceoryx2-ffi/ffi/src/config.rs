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

// BEGIN type definition

// NOTE check the README.md for using opaque types with renaming
/// The immutable pointer to the underlying `Config`
pub type iox2_config_ptr = *const Config;
/// The mutable pointer to the underlying `Config`
pub type iox2_config_mut_ptr = *mut Config;

// END type definition

// BEGIN C API
#[no_mangle]
pub extern "C" fn iox2_config_global_config() -> iox2_config_ptr {
    Config::global_config()
}

// END C API
