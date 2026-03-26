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

/// Registers the annotated function to the inventory to be executed by the
/// test runner.
///
/// Combine with `#[ignore]` to skip the test at runtime:
///
/// ```ignore
/// #[inventory_test]
/// fn my_test() { ... }
///
/// #[ignore]
/// #[inventory_test]
/// fn my_ignored_test() { ... }
/// ```
pub fn proc_macro(
    _macro_parameters: proc_macro::TokenStream,
    test_function: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let test_function = parse_macro_input!(test_function as ItemFn);
    instantiate_inventory_test(&test_function, None).into()
}
