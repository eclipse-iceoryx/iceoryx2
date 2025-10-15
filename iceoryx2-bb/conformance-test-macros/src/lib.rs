// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

//! Attribute macro for defining a module containing conformance tests.
//!
//! This macro processes a module, collects all functions marked with
//! `#[conformance_test]`, and generates a new declarative macro named after the module.
//! The generated macro, when invoked, creates a test module with a test for each
//! conformance function, instantiated for each provided type.
//!
//! Have a look at the `How It Works` section of the how-to-write-conformance-tests.md
//! document for more details.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemMod};

/// Attribute macro for marking functions as conformance tests.
///
/// Functions marked with this attribute will be included in the generated
/// test suite when the module is processed by `conformance_test_module`.
#[proc_macro_attribute]
pub fn conformance_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Attribute macro for defining a conformance test module.
///
/// This macro scans a module for functions annotated with `#[conformance_test]` and generates a declarative
/// macro that, when invoked, will run all the conformance tests for the specified System Under Test (SUT)
/// types.
///
/// Have a look at the `How It Works` section of the how-to-write-conformance-tests.md
/// document for more details.
#[proc_macro_attribute]
pub fn conformance_test_module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemMod);
    let mod_ident = &input.ident;
    let macro_name = mod_ident;

    // collect all functions with #[conformance_test] attribute
    let conformance_test_fns = collect_conformance_test_functions(&input, mod_ident);

    // generate and append the declarative macro to the current module
    let output = quote! {
        #input

        #[macro_export]
        macro_rules! #macro_name {
            ($module_path:path, $($sut_type:ty),+) => {
                mod #mod_ident {
                    use $module_path::*;
                    #(#conformance_test_fns)*
                }
            };
        }
    };

    output.into()
}

/// Collects all functions marked with `#[conformance_test]` in a module.
///
/// For each such function, a test function is generated that calls the original
/// function, instantiated for each type provided to the generated macro.
///
/// # Arguments
///
/// * `module` - The module to scan for conformance test functions.
/// * `mod_ident` - The identifier of the module.
///
/// # Returns
///
/// A vector of token streams, each representing a generated test function.
fn collect_conformance_test_functions(
    module: &ItemMod,
    mod_ident: &syn::Ident,
) -> Vec<proc_macro2::TokenStream> {
    let mut conformance_test_fns = Vec::new();

    if let Some((_brace, items)) = &module.content {
        for item in items {
            if let syn::Item::Fn(func) = item {
                let fn_ident = &func.sig.ident;
                let fn_attrs = &func.attrs;

                // check if the function has #[conformance_test]
                if fn_attrs
                    .iter()
                    .any(|attr| attr.path().is_ident("conformance_test"))
                {
                    // collect all attributes except #[conformance_test]
                    let test_attrs = fn_attrs
                        .iter()
                        .filter(|attr| !attr.path().is_ident("conformance_test"))
                        .collect::<Vec<_>>();

                    // generate the new test function
                    conformance_test_fns.push(quote! {
                        #(#test_attrs)*
                        #[test]
                        fn #fn_ident() {
                            #mod_ident::#fn_ident::<$($sut_type),+>();
                        }
                    });
                }
            }
        }
    }

    conformance_test_fns
}
