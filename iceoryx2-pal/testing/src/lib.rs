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

//! Low-level testing utilities and helpers.
//!
//! These utilities may only leverage the platform abstraction
//! [`iceoryx2_pal_posix`] in their implementation. The `std` module may be
//! used in `std` builds, however a `no_std` equivalent must be included for
//! all usages.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

extern crate alloc;

#[macro_use]
pub mod assert;
pub mod lifetime_tracker;
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
            alloc::format!($($e),*),
            assert_that![color_end]
        )
    };
}

pub const AT_LEAST_TIMING_VARIANCE: f32 = iceoryx2_pal_configuration::AT_LEAST_TIMING_VARIANCE;

#[doc(hidden)]
pub fn is_terminal() -> bool {
    #[cfg(feature = "std")]
    {
        use std::io::IsTerminal;
        std::io::stderr().is_terminal()
    }
    #[cfg(all(not(feature = "std"), any(target_os = "linux", target_os = "nto",)))]
    {
        true
    }
    #[cfg(all(
        not(feature = "std"),
        not(any(target_os = "linux", target_os = "nto",))
    ))]
    {
        false
    }
}

#[doc(hidden)]
pub fn spin_until<F, G>(mut condition: F, _guard: G)
where
    F: FnMut() -> bool,
{
    loop {
        if condition() {
            break;
        }

        #[cfg(feature = "std")]
        {
            std::thread::yield_now();
            std::thread::sleep(core::time::Duration::from_millis(10));
            std::thread::yield_now();
        }

        #[cfg(not(feature = "std"))]
        core::hint::spin_loop();
    }
}
