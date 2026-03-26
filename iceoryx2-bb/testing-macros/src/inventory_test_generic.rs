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

use quote::quote;
use syn::{parse_macro_input, ItemFn};

use crate::inventory_test_common::{
    extract_should_ignore, extract_should_panic, generate_for_constexpr_type_pairs,
    generate_for_types, is_pair, strip_attributes,
};

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

    let should_ignore = extract_should_ignore(&test_function.attrs);
    let should_panic = extract_should_panic(&test_function.attrs);

    let (wrapper_functions, inventory_submissions) = if is_pair(&macro_parameters) {
        generate_for_constexpr_type_pairs(
            &test_function,
            &macro_parameters,
            should_ignore,
            &should_panic,
        )
    } else {
        generate_for_types(
            &test_function,
            &macro_parameters,
            should_ignore,
            &should_panic,
        )
    };

    let mut generated = vec![];
    generated.push(strip_attributes(&test_function));
    generated.push(wrapper_functions);
    generated.push(inventory_submissions);

    proc_macro::TokenStream::from(quote! { #(#generated)* })
}
