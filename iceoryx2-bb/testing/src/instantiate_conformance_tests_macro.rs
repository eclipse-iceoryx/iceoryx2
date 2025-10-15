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
/// It generates a call to the specified module, passing the provided SUT types as arguments to the macro
/// defined within that module.
///
/// # Parameters
///
/// - `$module_path`: The path to the parent module where the conformance test module is defined.
/// - `$($sut_type:ty),+`: A comma-separated list of one or more system-under-test (SUT) types.
#[macro_export]
macro_rules! instantiate_conformance_tests {
    ($module_path:path, $($sut_type:ty),+) => {
        $module_path!($module_path, $($sut_type),+);
    };
}
