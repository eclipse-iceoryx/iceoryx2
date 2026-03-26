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

//! Testing utilities and helpers.
//!
//! These utilities may leverage components in the building blocks layer of
//! the architecture.
//!
//! Components from [`iceoryx2_pal_testing`] are re-exported for convenience
//! of use in upper layers.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

extern crate alloc;

pub use iceoryx2_pal_testing::*;
pub mod instantiate_conformance_tests_macro;
pub mod lifetime_tracker;
pub mod test_harness;

pub use inventory;
#[cfg(feature = "std")]
pub use libtest_mimic;

/// Default number of test threads for the custom test harness.
///
/// Tests using [`iceoryx2_pal_testing::lifetime_tracker::LifetimeTracker`] rely on global mutable state that is not
/// safe to access concurrently. Serial execution is required unless the caller
/// explicitly overrides this via `--test-threads`.
#[cfg(feature = "std")]
pub const DEFAULT_TEST_THREADS: usize = 1;

pub struct TestCase {
    pub module: &'static str,
    pub name: &'static str,
    pub test_fn: fn(),
    pub should_ignore: bool,
    pub should_panic: bool,
    pub should_panic_message: Option<&'static str>,
}
inventory::collect!(TestCase);

pub mod internal {
    #[cfg(any(target_os = "linux", target_os = "nto"))]
    pub use iceoryx2_pal_posix::posix::abort;
    pub use iceoryx2_pal_print::cout;
    pub use iceoryx2_pal_print::coutln;
}
