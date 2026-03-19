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
use syn::{
    Attribute, Expr, ExprLit, Ident, ItemFn, Lit, MetaNameValue, ReturnType, Signature, Type,
};

#[derive(Clone)]
pub enum ShouldPanic {
    No,
    Yes(Option<String>),
}

pub fn extract_should_ignore(test_function_attributes: &[Attribute]) -> bool {
    test_function_attributes
        .iter()
        .any(|attr| attr.path().is_ident("ignore"))
}

pub fn extract_should_panic(test_function_attributes: &[Attribute]) -> ShouldPanic {
    let found = test_function_attributes
        .iter()
        .find(|attr| attr.path().is_ident("should_panic"));

    let Some(attr) = found else {
        return ShouldPanic::No;
    };

    // #[should_panic(expected = "message")]
    let message = attr
        .parse_args::<MetaNameValue>()
        .ok()
        .and_then(|name_value| {
            if !name_value.path.is_ident("expected") {
                return None;
            }

            if let Expr::Lit(ExprLit {
                lit: Lit::Str(expected_string),
                ..
            }) = name_value.value
            {
                Some(expected_string.value())
            } else {
                None
            }
        });

    ShouldPanic::Yes(message)
}

/// `FileName::max_len()` -> `"max_len"`, `{ 64 }` -> `"64"`
fn constexpr_identifier_string(constexpr: &TokenStream) -> String {
    let s = constexpr.to_string().replace(['{', '}', '(', ')', ' '], "");
    s.split("::").last().unwrap_or(&s).to_string()
}

/// "foo::Bar<u64>" -> "Bar_u64"
pub fn type_identifier_string(type_identifier_string: &str) -> String {
    type_identifier_string
        .split("::")
        .last()
        .unwrap_or(type_identifier_string)
        .chars()
        .filter_map(|c| match c {
            '<' | ';' => Some('_'),
            '>' | ' ' | ',' | '[' | ']' => None,
            c => Some(c),
        })
        .collect()
}

/// Strips attributes handled by the test framework from the provided test
/// function.
pub fn strip_attributes(test_function: &ItemFn) -> TokenStream {
    let mut test_function_clone = test_function.clone();
    test_function_clone
        .attrs
        .retain(|attr| !attr.path().is_ident("should_panic") && !attr.path().is_ident("ignore"));
    quote! {
        #[allow(dead_code)]
        #test_function_clone
    }
}

/// Generate the test display name for a generic test instantiation.
pub fn generate_test_name(
    test_function_identifier: &Ident,
    constexprs: &[TokenStream],
    type_name: &TokenStream,
) -> String {
    let type_name = type_name.to_string().replace(' ', "");
    if constexprs.is_empty() {
        format!("{}<{}>", test_function_identifier, type_name)
    } else {
        let constexpr_names: Vec<String> = constexprs
            .iter()
            .map(|c| c.to_string().replace(['{', '}', ' '], ""))
            .collect();
        format!(
            "{}<{}, {}>",
            test_function_identifier,
            constexpr_names.join(", "),
            type_name
        )
    }
}

/// Generate the identifier for the wrapper function that calls the test
/// function.
fn generate_wrapper_identifier(
    test_function_name: &Ident,
    constexprs: &[TokenStream],
    type_name: &TokenStream,
) -> Ident {
    let suffix = if constexprs.is_empty() {
        type_identifier_string(&type_name.to_string())
    } else {
        let constexpr_id = constexprs
            .iter()
            .map(constexpr_identifier_string)
            .collect::<Vec<_>>()
            .join("_");
        format!(
            "__{}__{}",
            constexpr_id,
            type_identifier_string(&type_name.to_string())
        )
    };
    let ident_str = if suffix.is_empty() {
        format!("__inventory_test_{}", test_function_name)
    } else {
        format!("__inventory_test_{}_{}", test_function_name, suffix)
    };
    Ident::new(&ident_str, test_function_name.span())
}

// Generate the body of the wrapper function that calls the test function.
fn generate_wrapper_body(
    test_function_signature: &Signature,
    generic_parameters: Option<Vec<TokenStream>>,
) -> TokenStream {
    let identifier = &test_function_signature.ident;
    let f = if let Some(generic) = generic_parameters {
        quote! { #identifier::<#(#generic),*>() }
    } else {
        quote! { #identifier() }
    };

    if returns_result(test_function_signature) {
        quote! {
            if let Err(e) = #f {
                panic!("Test failed: {:?}", e);
            }
        }
    } else {
        f
    }
}

/// Generate a wrapper function that calls the test function.
///
/// Returns the wrapper function identifier alongside the generated function so
/// it may be used in an inventory submission.
pub fn generate_wrapper_function(
    test_function_signature: &Signature,
    constexprs: &[TokenStream],
    type_name: &TokenStream,
    generic_parameters: Option<Vec<TokenStream>>,
) -> (Ident, TokenStream) {
    let identifier =
        generate_wrapper_identifier(&test_function_signature.ident, constexprs, type_name);
    let body = generate_wrapper_body(test_function_signature, generic_parameters);
    let function = quote! {
        #[allow(non_snake_case, dead_code)]
        fn #identifier() {
            #body
        }
    };

    (identifier, function)
}

/// Generate an inventory submission for a test.
pub fn generate_inventory_submission(
    test_name: String,
    should_panic: ShouldPanic,
    should_ignore: bool,
    wrapper_name: &Ident,
) -> TokenStream {
    let (should_panic, should_panic_message) = match should_panic {
        ShouldPanic::No => (quote! { false }, quote! { None }),
        ShouldPanic::Yes(None) => (quote! { true }, quote! { None }),
        ShouldPanic::Yes(Some(msg)) => (quote! { true }, quote! { Some(#msg) }),
    };

    quote! {
        ::iceoryx2_bb_testing::inventory::submit! {
            ::iceoryx2_bb_testing::TestCase {
                name: #test_name,
                test_fn: #wrapper_name,
                should_ignore: #should_ignore,
                should_panic: #should_panic,
                should_panic_message: #should_panic_message,
            }
        }
    }
}

/// Check if function returns a Result type
fn returns_result(test_function_signature: &Signature) -> bool {
    match &test_function_signature.output {
        ReturnType::Type(_, ty) => match ty.as_ref() {
            Type::Path(type_path) => type_path
                .path
                .segments
                .last()
                .map(|seg| seg.ident == "Result")
                .unwrap_or(false),
            _ => false,
        },
        ReturnType::Default => false,
    }
}
