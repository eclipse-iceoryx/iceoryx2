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

use proc_macro2::{Delimiter, TokenStream, TokenTree};
use quote::quote;
use syn::{
    Attribute, Expr, ExprLit, Ident, ItemFn, Lit, MetaNameValue, ReturnType, Signature, Type,
};

pub const TEST_ATTRIBUTE: &str = "test";
pub const STRIPPED_ATTRIBUTES: &[&str] = &["should_panic", "ignore", TEST_ATTRIBUTE];

#[derive(Clone)]
pub enum ShouldPanic {
    No,
    Yes(Option<String>),
}

/// Generate tokens to instantiate tests and associated submission to the inventory.
pub fn instantiate_tests(
    test_function: &ItemFn,
    macro_parameters: Option<&TokenStream>,
) -> TokenStream {
    let should_ignore = extract_should_ignore(&test_function.attrs);
    let should_panic = extract_should_panic(&test_function.attrs);
    let stripped_test_function = strip_attributes(test_function);

    match macro_parameters {
        None => {
            let (wrapper_function, inventory_submission) =
                generate_standalone(test_function, should_ignore, &should_panic);

            quote! { #stripped_test_function #wrapper_function #inventory_submission }
        }
        Some(params) => {
            let (wrapper_functions, inventory_submissions) = if is_pair(params) {
                generate_for_constexpr_type_pair(
                    test_function,
                    params,
                    should_ignore,
                    &should_panic,
                )
            } else {
                generate_for_type(test_function, params, should_ignore, &should_panic)
            };

            quote! { #stripped_test_function #wrapper_functions #inventory_submissions }
        }
    }
}

/// Generate the test display name for a generic test instantiation.
pub fn generate_test_name(
    test_function_identifier: &Ident,
    constexprs: &[TokenStream],
    type_identifier: &TokenStream,
) -> String {
    let type_string = type_identifier.to_string().replace(' ', "");
    if constexprs.is_empty() {
        format!("<{}>::{}", type_string, test_function_identifier)
    } else {
        let constexpr_names: Vec<String> = constexprs
            .iter()
            .map(|c| c.to_string().replace(['{', '}', ' '], ""))
            .collect();
        format!(
            "<{}, {}>::{}",
            constexpr_names.join(", "),
            type_string,
            test_function_identifier
        )
    }
}

/// Generate a wrapper function that calls the test function.
///
/// Returns the wrapper function identifier alongside the generated function so
/// it may be used in an inventory submission.
pub fn generate_wrapper_function(
    test_function_signature: &Signature,
    constexpr_identifiers: &[TokenStream],
    type_identifier: &TokenStream,
    generic_parameters: Option<Vec<TokenStream>>,
) -> (Ident, TokenStream) {
    let identifier = generate_wrapper_identifier(
        &test_function_signature.ident,
        constexpr_identifiers,
        type_identifier,
    );
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
    wrapper_identifier: &Ident,
) -> TokenStream {
    let (should_panic, should_panic_message) = match should_panic {
        ShouldPanic::No => (quote! { false }, quote! { None }),
        ShouldPanic::Yes(None) => (quote! { true }, quote! { None }),
        ShouldPanic::Yes(Some(msg)) => (quote! { true }, quote! { Some(#msg) }),
    };

    quote! {
        ::iceoryx2_bb_testing::inventory::submit! {
            ::iceoryx2_bb_testing::TestCase {
                module: module_path!(),
                name: #test_name,
                test_fn: #wrapper_identifier,
                should_ignore: #should_ignore,
                should_panic: #should_panic,
                should_panic_message: #should_panic_message,
            }
        }
    }
}

/// Generate the identifier for the wrapper function that calls the test
/// function.
fn generate_wrapper_identifier(
    test_function_name: &Ident,
    constexprs: &[TokenStream],
    type_identifier: &TokenStream,
) -> Ident {
    let suffix = if constexprs.is_empty() {
        type_identifier_string(&type_identifier.to_string())
    } else {
        let constexpr_id = constexprs
            .iter()
            .map(constexpr_identifier_string)
            .collect::<Vec<_>>()
            .join("_");
        format!(
            "__{}__{}",
            constexpr_id,
            type_identifier_string(&type_identifier.to_string())
        )
    };
    let ident_str = if suffix.is_empty() {
        format!("__iox2_test_{}", test_function_name)
    } else {
        format!("__iox2_test_{}_{}", test_function_name, suffix)
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

/// Generate a wrapper and inventory submission for a single non-generic test
/// function.
fn generate_standalone(
    test_function: &ItemFn,
    should_ignore: bool,
    should_panic: &ShouldPanic,
) -> (TokenStream, TokenStream) {
    let (wrapper_identifier, wrapper_function) =
        generate_wrapper_function(&test_function.sig, &[], &TokenStream::new(), None);
    let inventory_submission = generate_inventory_submission(
        test_function.sig.ident.to_string(),
        should_panic.clone(),
        should_ignore,
        &wrapper_identifier,
    );
    (wrapper_function, inventory_submission)
}

/// Generate wrappers and inventory submissions for a test function instantiated
/// for each type in a comma-separated list.
fn generate_for_type(
    test_function: &ItemFn,
    macro_parameters: &TokenStream,
    should_ignore: bool,
    should_panic: &ShouldPanic,
) -> (TokenStream, TokenStream) {
    let constexprs: &[TokenStream] = &[];

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

/// Generate wrappers and inventory submissions for a test function instantiated
/// for each `(constexpr, ..., Type)` pair in the parameter list.
fn generate_for_constexpr_type_pair(
    test_function: &ItemFn,
    macro_parameters: &TokenStream,
    should_ignore: bool,
    should_panic: &ShouldPanic,
) -> (TokenStream, TokenStream) {
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

fn extract_should_ignore(test_function_attributes: &[Attribute]) -> bool {
    test_function_attributes
        .iter()
        .any(|attr| attr.path().is_ident("ignore"))
}

fn extract_should_panic(test_function_attributes: &[Attribute]) -> ShouldPanic {
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

/// Strips attributes handled by the test framework from the provided test
/// function.
fn strip_attributes(test_function: &ItemFn) -> TokenStream {
    let mut test_function_clone = test_function.clone();
    test_function_clone
        .attrs
        .retain(|attr| !STRIPPED_ATTRIBUTES.iter().any(|s| attr.path().is_ident(s)));
    quote! {
        #[allow(dead_code)]
        #test_function_clone
    }
}

/// `FileName::max_len()` -> `"max_len"`, `{ 64 }` -> `"64"`
fn constexpr_identifier_string(constexpr: &TokenStream) -> String {
    let s = constexpr.to_string().replace(['{', '}', '(', ')', ' '], "");
    s.split("::").last().unwrap_or(&s).to_string()
}

/// "foo::Bar<u64>" -> "Bar_u64"
fn type_identifier_string(type_identifier_string: &str) -> String {
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

/// Returns true if the macro parameters are constexpr/type pairs, e.g.
/// `(64, MyType), (128, MyType)`.
fn is_pair(macro_parameters: &TokenStream) -> bool {
    if let Some(TokenTree::Group(g)) = macro_parameters.clone().into_iter().next() {
        g.delimiter() == Delimiter::Parenthesis
    } else {
        false
    }
}

/// `(a, b), (c, d)` -> [TokenStream("a, b"), TokenStream("c, d")]
fn split_groups(macro_parameters: &TokenStream) -> Vec<TokenStream> {
    macro_parameters
        .clone()
        .into_iter()
        .filter_map(|token| match token {
            TokenTree::Group(g) if g.delimiter() == Delimiter::Parenthesis => Some(g.stream()),
            _ => None,
        })
        .collect()
}

/// Split a token stream into parts at each top-level comma.
fn split_on_comma(stream: TokenStream) -> Vec<TokenStream> {
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
fn split_constexpr_and_type(pair: TokenStream) -> (Vec<TokenStream>, TokenStream) {
    let mut parts = split_on_comma(pair);

    if parts.len() < 2 {
        panic!("Expected format: (const_expr, Type); got fewer than 2 comma-separated items");
    }

    let ty = parts.pop().unwrap();
    let constexprs = parts;

    (constexprs, ty)
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
