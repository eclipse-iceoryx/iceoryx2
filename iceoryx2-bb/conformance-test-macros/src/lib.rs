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

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemMod};

#[proc_macro_attribute]
pub fn conformance_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn conformance_test_module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemMod);
    let mod_ident = &input.ident;

    // collect all functions with #[conformance_test] attribute
    let conformance_test_fns = collect_conformance_test_functions(&input, &mod_ident);

    let macro_name = syn::Ident::new(&format!("{}_tests", mod_ident), mod_ident.span());
    // generate the declarative macro
    let output = quote! {
        #input

        #[macro_export]
        macro_rules! #macro_name {
            ($service_type:ty) => { // TODO: replace $service_type:ty with the number of generics of the functions or try to figure out how variadics work with macros
                mod #mod_ident {
                    #(#conformance_test_fns)*
                }
            };
        }
    };

    // eprintln!("Generated code:\n{}", output);

    output.into()
}

/// Collect all functions with #[conformance_test] attribute
fn collect_conformance_test_functions(
    module: &ItemMod,
    mod_path: &syn::Ident,
) -> Vec<proc_macro2::TokenStream> {
    let mut conformance_test_fns = Vec::new();

    if let Some((_brace, items)) = &module.content {
        for item in items {
            if let syn::Item::Fn(func) = item {
                for attr in &func.attrs {
                    if attr.path().is_ident("conformance_test") {
                        let fn_ident = &func.sig.ident;
                        // TODO return the number of generics and create multiple macro parameter
                        // TODO use the number of generics to replace $service_type with $gen_par_1, $gen_par_2, etc
                        // let generics = &func.sig.generics;
                        // let num_generics = generics.params.len();

                        conformance_test_fns.push(quote! {
                                #[test]
                                fn #fn_ident() {
                                    super::super::#mod_path::#mod_path::#fn_ident::<
                                    $service_type
                                    >();
                                }
                        });
                    }
                }
            }
        }
    }

    conformance_test_fns
}
