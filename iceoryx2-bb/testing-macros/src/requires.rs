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
use syn::{Expr, ItemMod, parse_macro_input};

use crate::internal::generate_requirement_submission;

/// Registers a requirement for the tests of the module and its submodules.
///
/// The expression is evaluated inside the module when the tests are
/// collected, so paths in it are relative to the module, and it yields
/// `Result<(), &'static str>`. The tests are ignored with the reason when it
/// is `Err`.
///
/// ```no_run
/// use iceoryx2_bb_testing_macros::requires;
///
/// fn supported() -> Result<(), &'static str> {
///     Err("the feature is not available here")
/// }
///
/// #[requires(super::supported())]
/// mod my_tests {
///     #[iceoryx2_bb_testing_macros::test]
///     pub fn ignored_here() {}
/// }
///
/// fn main() {}
/// ```
pub fn proc_macro(
    macro_parameters: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let check = parse_macro_input!(macro_parameters as Expr);
    let module = parse_macro_input!(item as ItemMod);

    let vis = &module.vis;
    let mod_token = &module.mod_token;
    let ident = &module.ident;
    let attrs = &module.attrs;

    let Some((_, items)) = module.content else {
        return syn::Error::new_spanned(
            &module.ident,
            "#[requires] requires a module with an inline body",
        )
        .to_compile_error()
        .into();
    };

    let submission = generate_requirement_submission(&check);

    quote! {
        #(#attrs)*
        #vis #mod_token #ident {
            #(#items)*
            #submission
        }
    }
    .into()
}
