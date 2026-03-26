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

use syn::{parse_macro_input, ItemFn};

use crate::inventory_test_common::instantiate_inventory_test;

/// Registers the annotated generic function to the inventory for each provided
/// type, to be executed by the test runner.
///
/// Accepts a comma-separated list of types or constexpr/type pairs.
/// Combine with `#[ignore]` to skip all instantiations at runtime:
///
/// ```ignore
/// #[inventory_test_generic(u32, u64)]
/// fn my_test<T>() { ... }
///
/// #[ignore]
/// #[inventory_test_generic(u32, u64)]
/// fn my_ignored_test<T>() { ... }
///
/// #[inventory_test_generic((64, MyType), (128, MyType))]
/// fn my_pair_test<const N: usize, T>() { ... }
/// ```
pub fn proc_macro(
    macro_parameters: proc_macro::TokenStream,
    test_function: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let test_function = parse_macro_input!(test_function as ItemFn);
    let macro_parameters: proc_macro2::TokenStream = macro_parameters.into();
    instantiate_inventory_test(&test_function, Some(&macro_parameters)).into()
}
