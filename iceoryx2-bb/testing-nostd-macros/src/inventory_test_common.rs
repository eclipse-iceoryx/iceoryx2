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

pub fn parse_tokens(s: &str) -> proc_macro2::TokenStream {
    s.parse()
        .unwrap_or_else(|_| panic!("Failed to parse: {}", s))
}

/// Convert type spec to readable name for identifiers
/// E.g., "foo::Bar<u64>" -> "Bar_u64"
pub fn make_pretty_type_string(type_spec: &str) -> String {
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

/// Prepends attributes to the provided test function.
///
/// Allows for attributes to be transparently applied to test function so the
/// user does not need to consider if and when they should be applied.
pub fn prepend_attributes(test_fn: &syn::ItemFn) -> proc_macro2::TokenStream {
    quote! {
        #[allow(dead_code)]
        #test_fn
    }
}

/// Create a wrapper function identifier.
///
/// A wrapper function is used to enable instantiating generic tests multiple
/// times with different parameters.
pub fn generate_wrapper_identifier(fn_name: &syn::Ident, suffix: &str) -> syn::Ident {
    syn::Ident::new(
        &format!("__inventory_test_{}_{}", fn_name, suffix),
        fn_name.span(),
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
    fn_name: &syn::Ident,
    sig: &syn::Signature,
    generic_parameters: Option<Vec<proc_macro2::TokenStream>>,
    ignored: bool,
) -> proc_macro2::TokenStream {
    if ignored {
        return quote! {
            iceoryx2_pal_print::cerr!("[IGNORED] ");
        };
    }

    let call = if let Some(generic) = generic_parameters {
        quote! { #fn_name::<#(#generic),*>() }
    } else {
        quote! { #fn_name() }
    };

    if returns_result(sig) {
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
    wrapper_name: syn::Ident,
    wrapper_body: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        #[allow(non_snake_case, dead_code)]
        fn #wrapper_name() {
            #wrapper_body
        }

        ::iceoryx2_bb_testing_nostd::inventory::submit! {
            ::iceoryx2_bb_testing_nostd::TestCase {
                name: #test_name,
                test_fn: #wrapper_name,
            }
        }
    }
}

/// Check if function returns a Result type
fn returns_result(sig: &syn::Signature) -> bool {
    match &sig.output {
        syn::ReturnType::Type(_, ty) => match ty.as_ref() {
            syn::Type::Path(type_path) => type_path
                .path
                .segments
                .last()
                .map(|seg| seg.ident == "Result")
                .unwrap_or(false),
            _ => false,
        },
        syn::ReturnType::Default => false,
    }
}
