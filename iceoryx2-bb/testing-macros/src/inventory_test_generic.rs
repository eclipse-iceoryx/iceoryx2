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
    extract_should_ignore, extract_should_panic, generate_inventory_submission,
    generate_wrapper_body, parse_tokens, strip_attributes, test_name, wrapper_name, ShouldPanic,
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
    let test_function_attributes = &test_function.attrs;
    let macro_parameters: proc_macro2::TokenStream = macro_parameters.into();

    let should_ignore = extract_should_ignore(test_function_attributes);
    let should_panic = extract_should_panic(test_function_attributes);

    let mut generated = vec![];
    // Include the original function to be called by the wrapper
    generated.push(strip_attributes(&test_function));

    if is_pair(&macro_parameters) {
        generated.push(tests_for_pairs(
            &test_function,
            &macro_parameters,
            should_ignore,
            &should_panic,
        ));
    } else {
        generated.push(tests_for_types(
            &test_function,
            &macro_parameters,
            should_ignore,
            &should_panic,
        ));
    }

    proc_macro::TokenStream::from(quote! { #(#generated)* })
}

fn tests_for_types(
    test_function: &ItemFn,
    macro_parameters: &proc_macro2::TokenStream,
    should_ignore: bool,
    should_panic: &ShouldPanic,
) -> proc_macro2::TokenStream {
    let test_function_name = &test_function.sig.ident;
    let test_function_signature = &test_function.sig;

    let tests: Vec<proc_macro2::TokenStream> = extract_types(macro_parameters)
        .iter()
        .map(|type_name| {
            let constexprs: &[&str] = &[];

            let test_name = test_name(test_function_name, constexprs, type_name);
            let wrapper_name = wrapper_name(test_function_name, constexprs, type_name);
            let wrapper_body = generate_wrapper_body(
                test_function_name,
                test_function_signature,
                Some(vec![parse_tokens(type_name)]),
            );

            generate_inventory_submission(
                test_name,
                should_panic.clone(),
                should_ignore,
                wrapper_name,
                wrapper_body,
            )
        })
        .collect();

    quote! { #(#tests)* }
}

fn tests_for_pairs(
    test_function: &ItemFn,
    macro_parameters: &proc_macro2::TokenStream,
    should_ignore: bool,
    should_panic: &ShouldPanic,
) -> proc_macro2::TokenStream {
    let test_function_name = &test_function.sig.ident;
    let test_function_signature = &test_function.sig;

    let tests: Vec<proc_macro2::TokenStream> = parse_pairs(&macro_parameters.to_string())
        .into_iter()
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (constexprs, type_name) = split_constexpr_and_type(&pair);

            let test_name = test_name(test_function_name, &constexprs, type_name);
            let wrapper_name = wrapper_name(test_function_name, &constexprs, type_name);

            let wrapper_body = generate_wrapper_body(
                test_function_name,
                test_function_signature,
                Some(generic_tokens(&constexprs, type_name)),
            );

            generate_inventory_submission(
                test_name,
                should_panic.clone(),
                should_ignore,
                wrapper_name,
                wrapper_body,
            )
        })
        .collect();

    quote! { #(#tests)* }
}

fn is_pair(macro_parameters: &proc_macro2::TokenStream) -> bool {
    macro_parameters.to_string().trim().starts_with('(')
}

/// Parse parenthesis-delimited pairs from attribute string.
///
/// Each pair is converted into a comma separated string entry in the vector.
/// E.g., "(a, b), (c, d)" -> ["a, b", "c, d"]
fn parse_pairs(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    let mut in_pair = false;

    for ch in s.chars() {
        match ch {
            '(' => {
                depth += 1;
                if depth == 1 {
                    in_pair = true;
                    current.clear();
                } else {
                    current.push(ch);
                }
            }
            ')' => {
                depth -= 1;
                if depth == 0 {
                    in_pair = false;
                    result.push(current.trim().to_string());
                    current.clear();
                } else {
                    current.push(ch);
                }
            }
            ',' if depth == 0 => {
                // Comma between pairs, ignore
            }
            _ => {
                if in_pair {
                    current.push(ch);
                }
            }
        }
    }

    result
}

fn extract_types(macro_parameters: &proc_macro2::TokenStream) -> Vec<String> {
    macro_parameters
        .to_string()
        .split(',')
        .map(|s| s.trim().to_owned())
        .collect()
}

/// Split "const1, const2, Type" into (["const1", "const2"], "Type")
fn split_constexpr_and_type(pair: &str) -> (Vec<&str>, &str) {
    let parts: Vec<&str> = pair.split(',').map(|s| s.trim()).collect();

    if parts.len() < 2 {
        panic!("Expected format: (const_expr, Type); got: ({})", pair);
    }

    let type_part = parts.last().unwrap();
    let generic_parts = parts[..parts.len() - 1].to_vec();

    (generic_parts, type_part)
}

/// Collect constexpr and type strings into a list of token streams for code generation.
fn generic_tokens(constexprs: &[&str], type_name: &str) -> Vec<proc_macro2::TokenStream> {
    let mut tokens = Vec::new();
    for c in constexprs {
        tokens.push(parse_tokens(c));
    }
    tokens.push(parse_tokens(type_name));
    tokens
}

