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
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Attribute, Expr, ExprLit, GenericArgument, Ident, ItemFn, Lit, MetaNameValue,
    ReturnType, Signature, Token, Type,
};

enum MacroParameters {
    Types(Punctuated<GenericArgument, Token![,]>),
    ConstexprTypePairs(Vec<(GenericArgument, GenericArgument)>),
}

impl Parse for MacroParameters {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(token::Paren) {
            let mut pairs = Vec::new();

            while !input.is_empty() {
                let content;
                syn::parenthesized!(content in input);
                let constexpr = content.parse::<GenericArgument>()?;
                let _: Token![,] = content.parse()?;
                let ty = content.parse::<GenericArgument>()?;
                pairs.push((constexpr, ty));

                if input.peek(Token![,]) {
                    let _: Token![,] = input.parse()?;
                }
            }

            Ok(MacroParameters::ConstexprTypePairs(pairs))
        } else {
            Ok(MacroParameters::Types(Punctuated::parse_terminated(input)?))
        }
    }
}

enum TestGenerics<'a> {
    None,
    Type(&'a GenericArgument),
    ConstexprAndType(&'a GenericArgument, &'a GenericArgument),
}

pub(crate) enum RunMode {
    Normal,
    Ignore(Option<String>),
    ExpectPanic(Option<String>),
}

pub(crate) enum RequiresStd {
    No,
    Yes(Option<String>),
}

impl ToTokens for RunMode {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            Self::Normal => quote! { ::iceoryx2_bb_testing::RunMode::Normal },
            Self::Ignore(None) => quote! { ::iceoryx2_bb_testing::RunMode::Ignore(None) },
            Self::Ignore(Some(reason)) => {
                quote! { ::iceoryx2_bb_testing::RunMode::Ignore(Some(#reason)) }
            }
            Self::ExpectPanic(None) => {
                quote! { ::iceoryx2_bb_testing::RunMode::ExpectPanic(None) }
            }
            Self::ExpectPanic(Some(message)) => {
                quote! { ::iceoryx2_bb_testing::RunMode::ExpectPanic(Some(#message)) }
            }
        });
    }
}

impl ToTokens for RequiresStd {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            Self::No => quote! { ::iceoryx2_bb_testing::RequiresStd::No },
            Self::Yes(None) => quote! { ::iceoryx2_bb_testing::RequiresStd::Yes(None) },
            Self::Yes(Some(reason)) => {
                quote! { ::iceoryx2_bb_testing::RequiresStd::Yes(Some(#reason)) }
            }
        });
    }
}

/// Generate tokens to instantiate tests and associated submission to the inventory.
pub fn instantiate_tests(original_function: &ItemFn, macro_parameters: TokenStream) -> TokenStream {
    let parameters = (!macro_parameters.is_empty()).then(|| {
        syn::parse2::<MacroParameters>(macro_parameters).expect("failed to parse macro parameters")
    });

    let run_mode = extract_run_mode(&original_function.attrs);
    let requires_std = extract_requires_std(&original_function.attrs);
    let test_function = generate_test_function(original_function);

    match parameters.as_ref() {
        None => {
            let (wrapper_function, inventory_submission) =
                generate_standalone(original_function, &run_mode, &requires_std);

            quote! { #test_function #wrapper_function #inventory_submission }
        }
        Some(MacroParameters::Types(types)) => {
            let (wrapper_functions, inventory_submissions) =
                generate_for_types(original_function, types, &run_mode, &requires_std);

            quote! { #test_function #wrapper_functions #inventory_submissions }
        }
        Some(MacroParameters::ConstexprTypePairs(pairs)) => {
            let (wrapper_functions, inventory_submissions) = generate_for_constexpr_type_pairs(
                original_function,
                pairs,
                &run_mode,
                &requires_std,
            );

            quote! { #test_function #wrapper_functions #inventory_submissions }
        }
    }
}

/// Generate a wrapper and inventory submission for a single non-generic test
/// function.
fn generate_standalone(
    original_function: &ItemFn,
    run_mode: &RunMode,
    requires_std: &RequiresStd,
) -> (TokenStream, TokenStream) {
    let (wrapper_identifier, wrapper_function) =
        generate_wrapper_function(original_function, TestGenerics::None);

    let attributes = strip_requires_std(&strip_test_attributes(&original_function.attrs));
    let inventory_submission = generate_inventory_submission(
        original_function.sig.ident.to_string(),
        &wrapper_identifier,
        &attributes,
        run_mode,
        requires_std,
    );
    (wrapper_function, inventory_submission)
}

/// Generate wrappers and inventory submissions for a test function instantiated
/// for each type in a comma-separated list.
fn generate_for_types(
    original_function: &ItemFn,
    types: &Punctuated<GenericArgument, Token![,]>,
    run_mode: &RunMode,
    requires_std: &RequiresStd,
) -> (TokenStream, TokenStream) {
    let attributes = strip_requires_std(&strip_test_attributes(&original_function.attrs));

    let (wrapper_functions, inventory_submissions): (Vec<_>, Vec<_>) = types
        .iter()
        .map(|ty| {
            let (wrapper_identifier, wrapper_function) =
                generate_wrapper_function(original_function, TestGenerics::Type(ty));
            let inventory_submission = generate_inventory_submission(
                generate_test_name(&original_function.sig.ident, TestGenerics::Type(ty)),
                &wrapper_identifier,
                &attributes,
                run_mode,
                requires_std,
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
/// for each `(constexpr, Type)` pair in the parameter list.
fn generate_for_constexpr_type_pairs(
    test_function: &ItemFn,
    pairs: &[(GenericArgument, GenericArgument)],
    run_mode: &RunMode,
    requires_std: &RequiresStd,
) -> (TokenStream, TokenStream) {
    let attributes = strip_requires_std(&strip_test_attributes(&test_function.attrs));

    let (wrapper_functions, inventory_submissions): (Vec<_>, Vec<_>) = pairs
        .iter()
        .map(|(constexpr, ty)| {
            let (wrapper_identifier, wrapper_function) = generate_wrapper_function(
                test_function,
                TestGenerics::ConstexprAndType(constexpr, ty),
            );
            let inventory_submission = generate_inventory_submission(
                generate_test_name(
                    &test_function.sig.ident,
                    TestGenerics::ConstexprAndType(constexpr, ty),
                ),
                &wrapper_identifier,
                &attributes,
                run_mode,
                requires_std,
            );

            (wrapper_function, inventory_submission)
        })
        .unzip();

    (
        quote! { #(#wrapper_functions)* },
        quote! { #(#inventory_submissions)* },
    )
}

/// Generate the test display name for a generic test instantiation.
fn generate_test_name(original_identifier: &Ident, generics: TestGenerics<'_>) -> String {
    match generics {
        TestGenerics::Type(ty) => {
            format!("<{}>::{}", type_display_string(ty), original_identifier)
        }
        TestGenerics::ConstexprAndType(c, ty) => format!(
            "<{}, {}>::{}",
            constexpr_display_string(c),
            type_display_string(ty),
            original_identifier
        ),
        TestGenerics::None => unreachable!(),
    }
}

/// Generate a wrapper function that calls the test function.
///
/// Returns the wrapper function identifier alongside the generated function so
/// it may be used in an inventory submission.
fn generate_wrapper_function(
    original_function: &ItemFn,
    generics: TestGenerics<'_>,
) -> (Ident, TokenStream) {
    let attributes = strip_test_attributes(&original_function.attrs);
    let identifier = generate_wrapper_identifier(&original_function.sig.ident, &generics);
    let body = generate_wrapper_body(&original_function.sig, generics);
    let function = quote! {
        #[allow(non_snake_case, dead_code)]
        #(#attributes)*
        fn #identifier() {
            #body
        }
    };

    (identifier, function)
}

/// Generate the identifier for the wrapper function that calls the test
/// function.
fn generate_wrapper_identifier(original_identifier: &Ident, generics: &TestGenerics<'_>) -> Ident {
    let suffix = match generics {
        TestGenerics::None => String::new(),
        TestGenerics::Type(ty) => type_identifier_string(ty),
        TestGenerics::ConstexprAndType(c, ty) => format!(
            "__{}__{}",
            constexpr_identifier_string(c),
            type_identifier_string(ty)
        ),
    };

    let wrapper_identifier = if suffix.is_empty() {
        format!("__iox2_test_{}", original_identifier)
    } else {
        format!("__iox2_test_{}_{}", original_identifier, suffix)
    };

    Ident::new(&wrapper_identifier, original_identifier.span())
}

// Generate the body of the wrapper function that calls the test function.
fn generate_wrapper_body(
    original_signature: &Signature,
    generics: TestGenerics<'_>,
) -> TokenStream {
    let identifier = &original_signature.ident;
    let f = match generics {
        TestGenerics::None => quote! { #identifier() },
        TestGenerics::Type(ty) => quote! { #identifier::<#ty>() },
        TestGenerics::ConstexprAndType(c, ty) => quote! { #identifier::<#c, #ty>() },
    };

    if returns_result(original_signature) {
        quote! {
            if let Err(e) = #f {
                panic!("Test failed: {:?}", e);
            }
        }
    } else {
        f
    }
}

/// Generate an inventory submission for a test.
pub fn generate_inventory_submission(
    test_name: String,
    wrapper_identifier: &Ident,
    attributes: &[Attribute],
    run_mode: &RunMode,
    requires_std: &RequiresStd,
) -> TokenStream {
    quote! {
        #(#attributes)*
        ::iceoryx2_bb_testing::inventory::submit! {
            ::iceoryx2_bb_testing::TestCase {
                module: module_path!(),
                name: #test_name,
                test_fn: #wrapper_identifier,
                run_mode: #run_mode,
                requires_std: #requires_std,
            }
        }
    }
}

fn extract_run_mode(attrs: &[Attribute]) -> RunMode {
    let ignore = attrs.iter().find(|attr| attr.path().is_ident("ignore"));
    let should_panic = attrs
        .iter()
        .find(|attr| attr.path().is_ident("should_panic"));

    // #[ignore] takes priority over #[should_panic]
    if let Some(attr) = ignore {
        let reason = attr.parse_args::<Lit>().ok().and_then(|lit| {
            if let Lit::Str(s) = lit {
                Some(s.value())
            } else {
                None
            }
        });
        return RunMode::Ignore(reason);
    }

    if let Some(attr) = should_panic {
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
        return RunMode::ExpectPanic(message);
    }

    RunMode::Normal
}

fn extract_requires_std(attrs: &[Attribute]) -> RequiresStd {
    let Some(attr) = attrs
        .iter()
        .find(|attr| attr.path().is_ident("requires_std"))
    else {
        return RequiresStd::No;
    };
    let reason = attr.parse_args::<Lit>().ok().and_then(|lit| {
        if let Lit::Str(s) = lit {
            Some(s.value())
        } else {
            None
        }
    });
    RequiresStd::Yes(reason)
}

/// Strips test framework attributes from an attribute list.
fn strip_test_attributes(attrs: &[Attribute]) -> Vec<Attribute> {
    const TEST_ATTRIBUTES: &[&str] = &["test", "should_panic", "ignore"];

    attrs
        .iter()
        .filter(|attr| !TEST_ATTRIBUTES.iter().any(|s| attr.path().is_ident(s)))
        .cloned()
        .collect()
}

/// Strips `#[requires_std]` from an attribute list.
fn strip_requires_std(attrs: &[Attribute]) -> Vec<Attribute> {
    attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("requires_std"))
        .cloned()
        .collect()
}

/// Generates a copy of the original test function, stripping anything that is
/// not required in the test context
fn generate_test_function(original_function: &ItemFn) -> TokenStream {
    let attributes = strip_test_attributes(&original_function.attrs);
    let (vis, sig, block) = (
        &original_function.vis,
        &original_function.sig,
        &original_function.block,
    );

    quote! {
        #[allow(dead_code)]
        #(#attributes)*
        #vis #sig #block
    }
}

/// `FileName::max_len()` -> `"FileName::max_len"`, `{ 64 }` -> `"64"`
fn constexpr_display_string(constexpr: &GenericArgument) -> String {
    constexpr
        .to_token_stream()
        .to_string()
        .replace(['{', '}', '(', ')', ' '], "")
}

/// `FileName::max_len()` -> `"FileName_max_len"`, `{ 64 }` -> `"64"`
fn constexpr_identifier_string(constexpr: &GenericArgument) -> String {
    constexpr_display_string(constexpr).replace("::", "_")
}

/// Strips white-space present in the token stream.
///
/// `foo::Bar<u64>` -> `"foo::Bar<u64>"`, `[u8; 4]` -> `"[u8;4]"`
fn type_display_string(ty: &GenericArgument) -> String {
    ty.to_token_stream().to_string().replace(' ', "")
}

/// `foo::Bar<u64>` -> `"Bar_u64"`, `[u8; 4]` -> `"u8_4"`
fn type_identifier_string(ty: &GenericArgument) -> String {
    let s = ty.to_token_stream().to_string();
    s.split("::")
        .last()
        .unwrap_or(&s)
        .chars()
        .filter_map(|c| match c {
            '<' | ';' => Some('_'),
            '>' | ' ' | ',' | '[' | ']' => None,
            c => Some(c),
        })
        .collect()
}

/// Check if function returns a Result type
fn returns_result(original_signature: &Signature) -> bool {
    match &original_signature.output {
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
