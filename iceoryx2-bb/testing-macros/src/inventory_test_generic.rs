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

use proc_macro2::{Delimiter, TokenTree};
use quote::quote;
use syn::{parse_macro_input, ItemFn};

use crate::inventory_test_common::{
    extract_should_ignore, extract_should_panic, generate_inventory_submission, generate_test_name,
    generate_wrapper_function, strip_attributes, ShouldPanic,
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

fn generate_for_types(
    test_function: &ItemFn,
    macro_parameters: &proc_macro2::TokenStream,
    should_ignore: bool,
    should_panic: &ShouldPanic,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let constexprs: &[proc_macro2::TokenStream] = &[];

    let (wrapper_functions, inventory_submissions): (Vec<_>, Vec<_>) =
        split_on_comma(macro_parameters.clone())
            .into_iter()
            .map(|type_name| {
                let (wrapper_function_identifier, wrapper_function) = generate_wrapper_function(
                    &test_function.sig,
                    constexprs,
                    &type_name,
                    Some(vec![type_name.clone()]),
                );
                let inventory_submission = generate_inventory_submission(
                    generate_test_name(&test_function.sig.ident, constexprs, &type_name),
                    should_panic.clone(),
                    should_ignore,
                    &wrapper_function_identifier,
                );

                (wrapper_function, inventory_submission)
            })
            .unzip();

    (
        quote! { #(#wrapper_functions)* },
        quote! { #(#inventory_submissions)* },
    )
}

fn generate_for_constexpr_type_pairs(
    test_function: &ItemFn,
    macro_parameters: &proc_macro2::TokenStream,
    should_ignore: bool,
    should_panic: &ShouldPanic,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let (wrapper_functions, inventory_submissions): (Vec<_>, Vec<_>) =
        split_groups(macro_parameters)
            .into_iter()
            .map(|pair| {
                let (constexprs, ty) = split_constexpr_and_type(pair);
                let mut generic_params = Vec::new();
                for constexpr in &constexprs {
                    generic_params.push(constexpr.clone());
                }
                generic_params.push(ty.clone());

                let (wrapper_function_identifier, wrapper_function) = generate_wrapper_function(
                    &test_function.sig,
                    &constexprs,
                    &ty,
                    Some(generic_params),
                );
                let inventory_submission = generate_inventory_submission(
                    generate_test_name(&test_function.sig.ident, &constexprs, &ty),
                    should_panic.clone(),
                    should_ignore,
                    &wrapper_function_identifier,
                );

                (wrapper_function, inventory_submission)
            })
            .unzip();

    (
        quote! { #(#wrapper_functions)* },
        quote! { #(#inventory_submissions)* },
    )
}

fn is_pair(macro_parameters: &proc_macro2::TokenStream) -> bool {
    if let Some(TokenTree::Group(g)) = macro_parameters.clone().into_iter().next() {
        g.delimiter() == Delimiter::Parenthesis
    } else {
        false
    }
}

/// `(a, b), (c, d)` -> [TokenStream("a, b"), TokenStream("c, d")]
fn split_groups(macro_parameters: &proc_macro2::TokenStream) -> Vec<proc_macro2::TokenStream> {
    macro_parameters
        .clone()
        .into_iter()
        .filter_map(|token| match token {
            TokenTree::Group(g) if g.delimiter() == Delimiter::Parenthesis => Some(g.stream()),
            _ => None,
        })
        .collect()
}

/// Split a token stream into parts at each comma.
fn split_on_comma(stream: proc_macro2::TokenStream) -> Vec<proc_macro2::TokenStream> {
    let mut result = Vec::new();
    let mut current: Vec<TokenTree> = Vec::new();

    for token in stream {
        match token {
            TokenTree::Punct(ref p) if p.as_char() == ',' => {
                result.push(current.drain(..).collect());
            }
            other => current.push(other),
        }
    }

    if !current.is_empty() {
        result.push(current.into_iter().collect());
    }

    result
}

/// `const1, const2, Type` -> `([const1, const2], Type)`
fn split_constexpr_and_type(
    pair: proc_macro2::TokenStream,
) -> (Vec<proc_macro2::TokenStream>, proc_macro2::TokenStream) {
    let mut parts = split_on_comma(pair);

    if parts.len() < 2 {
        panic!("Expected format: (const_expr, Type); got fewer than 2 comma-separated items");
    }

    let ty = parts.pop().unwrap();
    let constexprs = parts;

    (constexprs, ty)
}
