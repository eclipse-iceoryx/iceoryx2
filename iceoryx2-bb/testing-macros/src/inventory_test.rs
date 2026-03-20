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

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

use crate::inventory_test_common::{
    extract_should_ignore, extract_should_panic, generate_inventory_submission,
    generate_wrapper_function, strip_attributes,
};

/// Registers the annotated function to the inventory to be executed by the
/// test runner.
///
/// Combine with `#[ignore]` to skip the test at runtime:
///
/// ```ignore
/// #[ignore]
/// #[inventory_test]
/// fn my_test() { ... }
///
/// #[ignore]
/// #[inventory_test]
/// fn my_ignored_test() { ... }
/// ```
#[allow(clippy::disallowed_types)]
pub fn proc_macro(
    _macro_parameters: proc_macro::TokenStream,
    test_function: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let test_function = parse_macro_input!(test_function as ItemFn);

    let should_ignore = extract_should_ignore(&test_function.attrs);
    let should_panic = extract_should_panic(&test_function.attrs);

    // Generate wrapper around the test function
    // This is required to handle test functions that e.g. return Result
    // so they can be handled by the test runner
    let (wrapper_identifier, wrapper_function) =
        generate_wrapper_function(&test_function.sig, &[], &TokenStream::new(), None);

    let inventory_submission = generate_inventory_submission(
        test_function.sig.ident.to_string(),
        should_panic,
        should_ignore,
        &wrapper_identifier,
    );

    let mut generated = vec![];
    generated.push(strip_attributes(&test_function));
    generated.push(wrapper_function);
    generated.push(inventory_submission);

    proc_macro::TokenStream::from(quote! { #(#generated)* })
}
