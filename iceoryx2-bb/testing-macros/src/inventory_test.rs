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

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

use crate::inventory_test_common::{
    generate_inventory_submission, generate_wrapper_body, generate_wrapper_identifier,
    prepend_attributes,
};

/// Registers the annotated function to the inventory to be executed by the
/// test runner.
///
/// Accepts an optional `ignore` parameter to skip the test at runtime:
///
/// ```ignore
/// #[inventory_test(ignore)]
/// fn my_test() { ... }
/// ```
#[allow(clippy::disallowed_types)]
pub fn proc_macro(attr: TokenStream, item: TokenStream) -> TokenStream {
    let original_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &original_fn.sig.ident;

    let ignored = attr.to_string().split(',').any(|s| s.trim() == "ignore");

    let mut generated = vec![prepend_attributes(&original_fn)];

    // Generate wrapper function
    let wrapper_name = generate_wrapper_identifier(fn_name, "");
    let wrapper_body = generate_wrapper_body(fn_name, &original_fn.sig, None, ignored);

    // Generate inventory submission
    let submission = generate_inventory_submission(fn_name.to_string(), wrapper_name, wrapper_body);
    generated.push(submission);

    TokenStream::from(quote! { #(#generated)* })
}
