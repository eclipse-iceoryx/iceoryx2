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
use syn::parse::Parser;

/// Conditionally compiles in the function body based on the `std` feature.
/// When `std` is not enabled, the body is replaced with a skip message.
///
/// Accepts an optional reason string to explain why `std` is required:
///
/// ```ignore
/// #[requires_std]
/// fn my_test() { ... }
///
/// #[requires_std("threading")]
/// fn my_other_test() { ... }
/// ```
pub fn proc_macro(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item: syn::ItemFn = syn::parse2(item.into()).unwrap();
    let ident = &item.sig.ident;
    let attrs = &item.attrs;
    let vis = &item.vis;
    let generics = &item.sig.generics;
    let inputs = &item.sig.inputs;
    let output = &item.sig.output;
    let block = &item.block;

    let message = if attr.is_empty() {
        "[SKIPPED - requires std] ".to_string()
    } else {
        let parser = syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated;
        let reasons = parser.parse(attr).unwrap();
        let reason_strs: Vec<String> = reasons.iter().map(|lit| lit.value()).collect();
        format!("[SKIPPED - requires std: {}] ", reason_strs.join(", "))
    };

    // Detect test signatures with "Result" return type
    // Return Ok() for those, otherwise return nothing
    let noop = match &item.sig.output {
        syn::ReturnType::Default => quote! {},
        syn::ReturnType::Type(_, ty) => {
            if let syn::Type::Path(type_path) = ty.as_ref() {
                if type_path
                    .path
                    .segments
                    .first()
                    .is_some_and(|s| s.ident == "Result")
                {
                    quote! { Ok(()) }
                } else {
                    quote! {}
                }
            } else {
                quote! {}
            }
        }
    };

    quote! {
        #[allow(unused_variables)]
        #(#attrs)*
        #vis fn #ident #generics(#inputs) #output {
            #[cfg(feature = "std")]
            #block

            #[cfg(not(feature = "std"))]
            {
                iceoryx2_pal_print::cerr!(#message);
                #noop
            }
        }
    }
    .into()
}
