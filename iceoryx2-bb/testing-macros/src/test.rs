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

use crate::internal::instantiate_tests;

/// Registers the annotated function to the inventory to be executed by the
/// test runner.
///
/// ```no_run
/// use iceoryx2_bb_testing_macros::test;
///
/// #[test]
/// fn my_test() { /* ... */ }
///
/// #[ignore]
/// #[test]
/// fn my_ignored_test() { /* ... */ }
/// ```
pub fn proc_macro(
    macro_parameters: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let test_function = parse_macro_input!(item as ItemFn);
    let macro_parameters: proc_macro2::TokenStream = macro_parameters.into();
    let params = (!macro_parameters.is_empty()).then_some(&macro_parameters);

    instantiate_tests(&test_function, params).into()
}
