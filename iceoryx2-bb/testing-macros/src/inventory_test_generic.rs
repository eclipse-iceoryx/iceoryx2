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
    generate_wrapper_body, generate_wrapper_identifier, parse_tokens, strip_attributes,
    type_string, ShouldPanic,
};

/// Registers the annotated generic function to the inventory for each provided
/// type, to be executed by the test runner.
///
/// Accepts a comma-separated list of types or constexpr/type pairs, and an
/// optional `ignore` parameter to skip all instantiations at runtime:
///
/// ```ignore
/// #[inventory_test_generic(u32, u64)]
/// fn my_test<T>() { ... }
///
/// #[inventory_test_generic(ignore, u32, u64)]
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

    let should_ignore = extract_should_ignore(&macro_parameters);
    let should_panic = extract_should_panic(&test_function.attrs);

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
        .map(|ty| {
            let type_tokens = parse_tokens(ty);
            let type_name = type_string(ty);

            let test_name = format!("{}_{}", test_function_name, type_name);
            let wrapper_name = generate_wrapper_identifier(test_function_name, &type_name);
            let wrapper_body = generate_wrapper_body(
                test_function_name,
                test_function_signature,
                Some(vec![type_tokens]),
                should_ignore,
            );

            generate_inventory_submission(
                test_name,
                should_panic.clone(),
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
            let (constexprs, ty) = split_constexpr_and_type(&pair);

            let constexpr_tokens: Vec<_> = constexprs.iter().map(|c| parse_tokens(c)).collect();
            let type_tokens = parse_tokens(ty);

            let constexpr_string = constexpr_string(&constexprs);
            let type_string = type_string(ty);
            let test_function_name_suffix = suffix_string(&constexpr_string, &type_string);

            // Name printed in test output, different from actual test function name
            let test_name = format!("{}_{}", test_function_name, test_function_name_suffix);
            let wrapper_name =
                generate_wrapper_identifier(test_function_name, &test_function_name_suffix);

            let mut generics = constexpr_tokens;
            generics.push(type_tokens);
            let wrapper_body = generate_wrapper_body(
                test_function_name,
                test_function_signature,
                Some(generics),
                should_ignore,
            );

            generate_inventory_submission(
                test_name,
                should_panic.clone(),
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
        .filter(|s| s != "ignore")
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

/// Convert constexpr parts to readable name
/// E.g., ["{FileName::max_len()}"] -> "max_len"
/// E.g., ["{Path::capacity()}"] -> "capacity"
/// E.g., ["64"] -> "64"
fn constexpr_string(constexpr_parts: &[&str]) -> String {
    constexpr_parts
        .iter()
        .map(|p| {
            let stripped = p.replace(['{', '}', '(', ')'], "").replace(' ', "");

            if let Some(last_part) = stripped.split("::").last() {
                last_part.to_string()
            } else {
                stripped
            }
        })
        .collect::<Vec<_>>()
        .join("_")
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

/// Combine constexpr and type names with double underscores around const
/// E.g., ("max_len", "FileName") -> "__max_len__FileName"
/// E.g., ("", "FileName") -> "FileName"
fn suffix_string(constexpr: &str, ty: &str) -> String {
    if constexpr.is_empty() {
        ty.to_string()
    } else {
        format!("__{}__{}", constexpr, ty)
    }
}
