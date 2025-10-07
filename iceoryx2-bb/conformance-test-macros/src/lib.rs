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

    let macro_name = syn::Ident::new(&format!("{}", mod_ident), mod_ident.span());

    // generate the declarative macro
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

fn collect_conformance_test_functions(
    module: &ItemMod,
    mod_ident: &syn::Ident,
) -> Vec<proc_macro2::TokenStream> {
    let mut conformance_test_fns = Vec::new();

    if let Some((_brace, items)) = &module.content {
        for item in items {
            if let syn::Item::Fn(func) = item {
                for attr in &func.attrs {
                    if attr.path().is_ident("conformance_test") {
                        let fn_ident = func.sig.ident.clone();
                        conformance_test_fns.push(quote! {
                            #[test]
                            fn #fn_ident() {
                                #mod_ident::#fn_ident::<$($sut_type),+>();
                            }
                        });
                    }
                }
            }
        }
    }

    conformance_test_fns
}
