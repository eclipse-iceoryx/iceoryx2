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
use iceoryx2_bb_container::semantic_string::SemanticStringError;
use iceoryx2_bb_log::set_log_level;

use core::ffi::c_int;

mod node;
mod node_builder;
mod node_name;
mod publisher;
mod subscriber;

pub use node::*;
pub use node_builder::*;
pub use node_name::*;
pub use publisher::*;
pub use subscriber::*;

/// This constant signals an successful function call
pub const IOX2_OK: c_int = 0;

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_semantic_string_error_e {
    INVALID_CONTENT = IOX2_OK as isize + 1,
    EXCEEDS_MAXIMUM_LENGTH,
}

impl IntoCInt for SemanticStringError {
    fn into_c_int(self) -> c_int {
        (match self {
            SemanticStringError::InvalidContent => iox2_semantic_string_error_e::INVALID_CONTENT,
            SemanticStringError::ExceedsMaximumLength => {
                iox2_semantic_string_error_e::EXCEEDS_MAXIMUM_LENGTH
            }
        }) as c_int
    }
}

#[no_mangle]
pub extern "C" fn zero_copy_service_list() -> i32 {
    set_log_level(iceoryx2_bb_log::LogLevel::Info);

    let services = zero_copy::Service::list(Config::get_global_config());

    if services.is_err() {
        return -1;
    }

    let services = services.unwrap();

    for service in services {
        println!("\n{:#?}", &service);
    }

    0
}

/// This is a trait to convert a Rust error enum into the corresponding C error enum and then to a c_int in one go
///
/// # Example
///
/// ```no_run
/// use core::ffi::c_int;
///
/// trait IntoCInt {
///     fn into_c_int(self) -> c_int;
/// }
///
/// enum FooError {
///     BAR,
///     BAZ
/// }
///
/// #[repr(C)]
/// #[derive(Copy, Clone)]
/// pub enum iox2_foo_error_e {
///     BAR = 1, // start at 1 since IOX2_OK is already 0
///     BAZ,
/// }
///
/// impl IntoCInt for FooError {
///     fn into_c_int(self) -> c_int {
///         (match self {
///             FooError::BAR => iox2_foo_error_e::BAR,
///             FooError::BAZ => iox2_foo_error_e::BAZ,
///         }) as c_int
///     }
/// }
/// ```
trait IntoCInt {
    fn into_c_int(self) -> c_int;
}
