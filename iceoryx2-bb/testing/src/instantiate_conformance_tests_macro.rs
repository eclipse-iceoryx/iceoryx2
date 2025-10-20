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

#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

/// This macro is used to instantiate conformance tests for a given set of system-under-test (SUT) types.
///
/// It generates a call to the specified conformance test, passing the provided SUT types as arguments
/// to the macro defined within the conformance test module.
///
/// NOTE: If the same conformance test need to be instantiated multiple times in the same file, the
/// macro needs to be wrapped in a Rust module for each instantiation. For this case, it is recommended
/// to use the `instantiate_conformance_tests_with_module`.
///
/// # Parameters
///
/// - `$conformance_test_module_path`: The path to the parent module where the conformance test module is defined.
/// - `$($sut_type:ty),+`: A comma-separated list of one or more system-under-test (SUT) types.
#[macro_export]
macro_rules! instantiate_conformance_tests {
    ($conformance_test_module_path:path, $($sut_type:ty),+) => {
        $conformance_test_module_path!($conformance_test_module_path, $($sut_type),+);
    };
}

/// This is a convenience macro for `instantiate_conformance_tests`, to instantiate conformance tests
/// for a given set of system-under-test (SUT) types and automatically wrap them in a module.
///
/// It generates a call to the specified conformance test, passing the provided SUT types as arguments
/// to the macro defined within the conformance test module.
///
/// NOTE: If multiple conformance tests need to be instantiated in the same file, the `instantiate_conformance_tests`
/// macro can be used instead to simplify the handling.
///
/// # Parameters
///
/// - `$module_name`: The name of the module the conformance test shall be instantiated.
/// - `$conformance_test_module_path`: The path to the parent module where the conformance test module is defined.
/// - `$($sut_type:ty),+`: A comma-separated list of one or more system-under-test (SUT) types.
#[macro_export]
macro_rules! instantiate_conformance_tests_with_module {
    ($module_name:ident, $conformance_test_module_path:path, $($sut_type:ty),+) => {
        mod $module_name {
            use super::*;

            iceoryx2_bb_testing::instantiate_conformance_tests!(
                $conformance_test_module_path,
                $($sut_type),+
            );
        }
    };
}
