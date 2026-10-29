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
pub mod abandon_tracker;
pub mod allocator;
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

pub enum RunMode {
    Normal,
    Ignore(Option<&'static str>),
    ExpectPanic(Option<&'static str>),
}

pub enum RequiresStd {
    No,
    Yes(Option<&'static str>),
}

pub struct TestCase {
    pub module: &'static str,
    pub name: &'static str,
    pub test_fn: fn(),
    pub run_mode: RunMode,
    pub requires_std: RequiresStd,
}
inventory::collect!(TestCase);

/// A condition the tests of a module and its submodules need to run.
pub struct Requirement {
    pub module: &'static str,
    /// Returns OK or the reason the requirement is not met.
    pub check: fn() -> Result<(), &'static str>,
}
inventory::collect!(Requirement);

/// The reason the tests of `module` cannot run, if any.
///
/// The requirements registered on the module and on its parent modules are
/// checked, and the reason of the first one that is not met is returned.
pub fn unmet_requirement(module: &str) -> Option<&'static str> {
    inventory::iter::<Requirement>()
        .filter(|requirement| {
            module == requirement.module
                || module
                    .strip_prefix(requirement.module)
                    .is_some_and(|rest| rest.starts_with("::"))
        })
        .find_map(|requirement| (requirement.check)().err())
}

/// The modules whose requirement is not met, each with the reason, sorted by
/// module.
pub fn unmet_requirements() -> alloc::vec::Vec<(&'static str, &'static str)> {
    let mut unmet = inventory::iter::<Requirement>()
        .filter_map(|requirement| {
            (requirement.check)()
                .err()
                .map(|reason| (requirement.module, reason))
        })
        .collect::<alloc::vec::Vec<_>>();
    unmet.sort_unstable();
    unmet
}

pub mod internal {
    #[cfg(any(target_os = "linux", target_os = "nto"))]
    pub use iceoryx2_pal_posix::posix::abort;
    pub use iceoryx2_pal_print::cout;
    pub use iceoryx2_pal_print::coutln;
}
