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
    make_pretty_type_string, parse_tokens, prepend_attributes,
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
pub fn proc_macro(attr: TokenStream, item: TokenStream) -> TokenStream {
    let original_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &original_fn.sig.ident;
    let attr_str = attr.to_string();

    // Strip out `ignore` flag before processing type/pair arguments
    let ignored = attr_str.split(',').any(|s| s.trim() == "ignore");
    let attr_str = attr_str
        .split(',')
        .filter(|s| s.trim() != "ignore")
        .collect::<Vec<_>>()
        .join(",");
    let attr_str = attr_str.trim();

    let is_pair = attr_str.trim_start().starts_with('(');

    let mut generated = vec![prepend_attributes(&original_fn)];

    if is_pair {
        generate_tests_for_pairs(&mut generated, fn_name, &original_fn.sig, attr_str, ignored);
    } else {
        generate_tests_for_types(&mut generated, fn_name, &original_fn.sig, attr_str, ignored);
    }

    TokenStream::from(quote! { #(#generated)* })
}

fn generate_tests_for_types(
    generated: &mut Vec<proc_macro2::TokenStream>,
    fn_name: &syn::Ident,
    sig: &syn::Signature,
    attr_str: &str,
    ignored: bool,
) {
    for type_str in attr_str.split(',').map(|s| s.trim()) {
        let type_tokens = parse_tokens(type_str);
        let type_name = make_pretty_type_string(type_str);

        // Generate wrapper function
        let test_name = format!("{}_{}", fn_name, type_name);
        let wrapper_name = generate_wrapper_identifier(fn_name, &type_name);
        let wrapper_body = generate_wrapper_body(fn_name, sig, Some(vec![type_tokens]), ignored);

        // Generate inventory submission
        let submission = generate_inventory_submission(test_name, wrapper_name, wrapper_body);
        generated.push(submission);
    }
}

fn generate_tests_for_pairs(
    generated: &mut Vec<proc_macro2::TokenStream>,
    fn_name: &syn::Ident,
    sig: &syn::Signature,
    attr_str: &str,
    ignored: bool,
) {
    let pairs = parse_pairs(attr_str);

    for pair in pairs {
        if pair.is_empty() {
            continue;
        }

        let (constexprs, ty) = split_constexpr_and_type(&pair);

        // Parse tokens for constexpr and type pair
        let constexpr_tokens: Vec<_> = constexprs.iter().map(|c| parse_tokens(c)).collect();
        let type_tokens = parse_tokens(ty);

        // Generate readable names for test identification
        let constexpr_pretty = make_pretty_constexpr_string(&constexprs);
        let type_pretty = make_pretty_type_string(ty);
        let full_suffix = make_suffix_string(&constexpr_pretty, &type_pretty);

        // Generate wrapper function
        let test_name = format!("{}_{}", fn_name, full_suffix);
        let wrapper_name = generate_wrapper_identifier(fn_name, &full_suffix);

        let mut generics = constexpr_tokens;
        generics.push(type_tokens);
        let wrapper_body = generate_wrapper_body(fn_name, sig, Some(generics), ignored);

        // Generate inventory submission
        let submission = generate_inventory_submission(test_name, wrapper_name, wrapper_body);
        generated.push(submission);
    }
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

/// Split "const1, const2, Type" into (["const1", "const2"], "Type")
fn split_constexpr_and_type(pair_content: &str) -> (Vec<&str>, &str) {
    let parts: Vec<&str> = pair_content.split(',').map(|s| s.trim()).collect();

    if parts.len() < 2 {
        panic!(
            "Expected format: (const_expr, Type); got: ({})",
            pair_content
        );
    }

    let type_part = parts.last().unwrap();
    let generic_parts = parts[..parts.len() - 1].to_vec();

    (generic_parts, type_part)
}

/// Convert constexpr parts to readable name
/// E.g., ["{FileName::max_len()}"] -> "max_len"
/// E.g., ["{Path::capacity()}"] -> "capacity"
/// E.g., ["64"] -> "64"
fn make_pretty_constexpr_string(constexpr_parts: &[&str]) -> String {
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
fn make_suffix_string(constexpr: &str, ty: &str) -> String {
    if constexpr.is_empty() {
        ty.to_string()
    } else {
        format!("__{}__{}", constexpr, ty)
    }
}
