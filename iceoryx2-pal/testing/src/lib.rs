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

#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

extern crate alloc;

#[macro_use]
pub mod assert;
pub mod memory;
pub mod watchdog;

#[macro_export(local_inner_macros)]
macro_rules! test_requires {
    { $condition:expr } => {
        if !$condition { return; }
    }
}

#[macro_export(local_inner_macros)]
macro_rules! test_fail {
    ($($e:expr),*) => {
        core::panic!(
            "test failed: {} {} {}",
            assert_that![color_start],
            std::format_args!($($e),*).to_string(),
            assert_that![color_end]
        )
    };
}

pub const AT_LEAST_TIMING_VARIANCE: f32 =
    iceoryx2_pal_configuration::settings::AT_LEAST_TIMING_VARIANCE;
