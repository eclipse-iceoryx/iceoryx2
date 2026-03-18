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

pub fn parse_tokens(s: &str) -> TokenStream {
    s.parse()
        .unwrap_or_else(|_| panic!("Failed to parse: {}", s))
}

/// Convert type spec to readable name for identifiers
/// E.g., "foo::Bar<u64>" -> "Bar_u64"
pub fn type_string(type_spec: &str) -> String {
    type_spec
        .split("::")
        .last()
        .unwrap_or(type_spec)
        .replace('<', "_")
        .replace('>', "")
        .replace([' ', ','], "")
        .replace(['[', ']'], "")
        .replace(';', "_")
}

pub fn extract_should_ignore(macro_parameters: &TokenStream) -> bool {
    macro_parameters
        .to_string()
        .split(',')
        .any(|s| s.trim() == "ignore")
}

#[derive(Clone)]
pub enum ShouldPanic {
    No,
    Yes(Option<String>),
}

pub fn extract_should_panic(attributes: &[Attribute]) -> ShouldPanic {
    let found = attributes
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

/// Generates attributes for the wrapper of the provided function.
///
/// Strips any attributes not used.
pub fn strip_attributes(test_function: &ItemFn) -> TokenStream {
    let mut test_function_clone = test_function.clone();
    test_function_clone
        .attrs
        .retain(|attr| !attr.path().is_ident("should_panic"));
    quote! {
        #[allow(dead_code)]
        #test_function_clone
    }
}

/// Create a wrapper function identifier.
///
/// A wrapper function is used to enable instantiating generic tests multiple
/// times with different parameters.
pub fn generate_wrapper_identifier(
    test_function_name: &Ident,
    test_funtion_name_suffix: &str,
) -> Ident {
    Ident::new(
        &format!(
            "__inventory_test_{}_{}",
            test_function_name, test_funtion_name_suffix
        ),
        test_function_name.span(),
    )
}

/// Generate the body of a test wrapper function.
///
/// If the test is generic, instantiates the test function with the provided
/// generic parameters. If not generic, the test function is called as-is.
///
/// When `ignored` is `true`, the body emits a skip message instead of calling
/// the test function.
pub fn generate_wrapper_body(
    test_function_name: &Ident,
    test_function_signature: &Signature,
    generic_parameters: Option<Vec<TokenStream>>,
    ignored: bool,
) -> TokenStream {
    if ignored {
        return quote! {
            iceoryx2_pal_print::cerr!("[IGNORED] ");
        };
    }

    let call = if let Some(generic) = generic_parameters {
        quote! { #test_function_name::<#(#generic),*>() }
    } else {
        quote! { #test_function_name() }
    };

    if returns_result(test_function_signature) {
        quote! {
            if let Err(e) = #call {
                panic!("Test failed: {:?}", e);
            }
        }
    } else {
        call
    }
}

/// Generate the inventory submission code for a test.
///
/// Creates a wrapper function that calls the test and submits it to the inventory system.
pub fn generate_inventory_submission(
    test_name: String,
    should_panic: ShouldPanic,
    wrapper_name: Ident,
    wrapper_body: TokenStream,
) -> TokenStream {
    let (should_panic, should_panic_message) = match should_panic {
        ShouldPanic::No => (quote! { false }, quote! { None }),
        ShouldPanic::Yes(None) => (quote! { true }, quote! { None }),
        ShouldPanic::Yes(Some(msg)) => (quote! { true }, quote! { Some(#msg) }),
    };

    quote! {
        #[allow(non_snake_case, dead_code)]
        fn #wrapper_name() {
            #wrapper_body
        }

        ::iceoryx2_bb_testing::inventory::submit! {
            ::iceoryx2_bb_testing::TestCase {
                name: #test_name,
                test_fn: #wrapper_name,
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
