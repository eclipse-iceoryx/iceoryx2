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
use syn::{parse_macro_input, Item, ItemMod};

use crate::internal::{instantiate_tests, TEST_ATTRIBUTE};

/// Generates inventory submissions for all `#[test]`-annotated functions in
/// the module.
///
/// Generic type parameters can be optionally provided. In this case, an
/// instantiation for every `#[test]`-annotated generic function in the module
/// for each type is generated. Non-annotated functions are passed through
/// unchanged.
///
/// Stand-alone tests
///
/// ```no_run
/// use iceoryx2_bb_testing_macros::tests;
///
/// #[tests(u32, u64)]
/// mod my_tests {
///     #[test]
///     pub fn size_is_nonzero<T: Sized>() {
///         assert!(core::mem::size_of::<T>() > 0);
///     }
///
///     #[test]
///     pub fn non_generic_test() {}
///
///     fn helper() -> u32 { 42 }
/// }
/// ```
///
/// Const+type pairs — each test is instantiated once per pair
///
/// ```no_run
/// use iceoryx2_bb_testing_macros::tests;
///
/// #[tests(
///     (4, [u8; 4]),
///     (8, [u8; 8])
/// )]
/// mod my_const_tests {
///     #[test]
///     pub fn size_matches_const<const N: usize, T: Sized>() {
///         assert_eq!(core::mem::size_of::<T>(), N);
///     }
/// }
/// ```
pub fn proc_macro(
    macro_parameters: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let module = parse_macro_input!(item as ItemMod);
    let macro_parameters: proc_macro2::TokenStream = macro_parameters.into();

    let vis = &module.vis;
    let mod_token = &module.mod_token;
    let ident = &module.ident;
    let attrs = &module.attrs;

    let Some((_, items)) = module.content else {
        return syn::Error::new_spanned(
            &module.ident,
            "#[tests] requires a module with an inline body",
        )
        .to_compile_error()
        .into();
    };

    let instantiations: Vec<TokenStream> = items
        .into_iter()
        .map(|item| match item {
            Item::Fn(ref test_function)
                if test_function
                    .attrs
                    .iter()
                    .any(|a| a.path().is_ident(TEST_ATTRIBUTE)) =>
            {
                let params =
                    (!test_function.sig.generics.params.is_empty()).then_some(&macro_parameters);
                instantiate_tests(test_function, params)
            }
            other => quote! { #other },
        })
        .collect();

    quote! {
        #(#attrs)*
        #vis #mod_token #ident {
            #(#instantiations)*
        }
    }
    .into()
}
