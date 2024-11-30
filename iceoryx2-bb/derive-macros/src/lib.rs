// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

//! Contains helper derive macros for iceoryx2.

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Expr, ExprLit, Fields, Lit};

/// Implements the [`iceoryx2_bb_elementary::placement_default::PlacementDefault`] trait when all
/// fields of the struct implement it.
///
/// ```
/// use iceoryx2_bb_derive_macros::PlacementDefault;
/// use iceoryx2_bb_elementary::placement_default::PlacementDefault;
/// use std::alloc::{alloc, dealloc, Layout};
///
/// #[derive(PlacementDefault)]
/// struct MyLargeType {
///     value_1: u64,
///     value_2: Option<usize>,
///     value_3: [u8; 10485760],
/// }
///
/// let layout = Layout::new::<MyLargeType>();
/// let raw_memory = unsafe { alloc(layout) } as *mut MyLargeType;
/// unsafe { MyLargeType::placement_default(raw_memory) };
///
/// unsafe { &mut *raw_memory }.value_3[123] = 31;
///
/// unsafe { core::ptr::drop_in_place(raw_memory) };
/// unsafe { dealloc(raw_memory.cast(), layout) };
/// ```
#[proc_macro_derive(PlacementDefault)]
pub fn placement_default_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let place_default_impl = match input.data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Named(ref fields_named) => {
                let field_inits = fields_named.named.iter().map(|f| {
                    let name = &f.ident;
                    quote! {
                        let field_address = core::ptr::addr_of_mut!((*ptr).#name);
                        PlacementDefault::placement_default(field_address);
                    }
                });

                quote! {
                    unsafe fn placement_default(ptr: *mut Self) {
                        #(#field_inits)*
                    }
                }
            }
            Fields::Unnamed(ref fields_unnamed) => {
                let field_inits = fields_unnamed.unnamed.iter().enumerate().map(|(i, _)| {
                    let index = syn::Index::from(i);
                    quote! {
                        let field_address = core::ptr::addr_of_mut!((*ptr).#index);
                        PlacementDefault::placement_default(field_address);
                    }
                });

                quote! {
                    unsafe fn placement_default(ptr: *mut Self) {
                        #(#field_inits)*
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    unsafe fn placement_default(ptr: *mut Self) {
                    }
                }
            }
        },
        _ => unimplemented!(),
    };

    let expanded = quote! {
        impl #impl_generics PlacementDefault for #name #ty_generics #where_clause {
            #place_default_impl
        }
    };

    TokenStream::from(expanded)
}

/// Implements the [`iceoryx2_bb_elementary::AsStringLiteral`] trait for enums to provide a string representation of each enum variant.
///
/// The string representation can be customized using the `CustomString` attribute, otherwise it will
/// convert the variant name to lowercase and replace underscores with spaces.
///
/// # Example
/// ```
/// use iceoryx2_bb_derive_macros::StringLiteral;
/// use iceoryx2_bb_elementary::AsStringLiteral;
///
/// #[derive(StringLiteral)]
/// enum MyEnum {
///     #[CustomString = "custom variant one"]
///     VariantOne,
///     VariantTwo,
/// }
///
/// let v1 = MyEnum::VariantOne;
/// assert_eq!(v1.as_str_literal(), "custom variant one");
///
/// let v2 = MyEnum::VariantTwo;
/// assert_eq!(v2.as_str_literal(), "variant two");
/// ```
#[proc_macro_derive(StringLiteral, attributes(CustomString))]
pub fn string_literal_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    // Generate implementation converting enums to a string representation
    let as_string_literal_impl = match input.data {
        Data::Enum(ref data_enum) => {
            let enum_to_string_mapping = data_enum.variants.iter().map(|variant| {
                let enum_name = &variant.ident;
                let enum_string_literal = variant
                    .attrs
                    .iter()
                    .find_map(|attr| {
                        if !attr.path().is_ident("CustomString") {
                            return None;
                        }
                        // Get the value of CustomString as a string literal
                        match attr.meta.require_name_value() {
                            Ok(meta) => match &meta.value {
                                Expr::Lit(ExprLit {
                                    lit: Lit::Str(lit), ..
                                }) => Some(Literal::string(&lit.value())),
                                _ => None,
                            },
                            _ => None,
                        }
                    })
                    .unwrap_or_else(|| {
                        // If no CustomString, generates default string literal in the form
                        // MyEnum::MyVariantName => 'my variant name'
                        let enum_string_literal = enum_name
                            .to_string()
                            .chars()
                            .fold(String::new(), |mut acc, c| {
                                if c.is_uppercase() && !acc.is_empty() {
                                    acc.push('_');
                                }
                                acc.push(c);
                                acc
                            })
                            .chars()
                            .map(|c| match c {
                                '_' => ' ',
                                c => c.to_ascii_lowercase(),
                            })
                            .collect::<String>();
                        Literal::string(&enum_string_literal)
                    });

                // Maps each enum variant to its string representation
                match &variant.fields {
                    Fields::Unit => {
                        quote! {
                            Self::#enum_name => #enum_string_literal
                        }
                    }
                    Fields::Unnamed(_) => {
                        quote! {
                            Self::#enum_name(..) => #enum_string_literal
                        }
                    }
                    Fields::Named(_) => {
                        quote! {
                            Self::#enum_name{..} => #enum_string_literal
                        }
                    }
                }
            });

            // Generate the mapping for the enum variant
            quote! {
                fn as_str_literal(&self) -> &'static str {
                    match self {
                        #(#enum_to_string_mapping,)*
                    }
                }
            }
        }
        _ => {
            // Does not work for non-enum types
            let err =
                syn::Error::new_spanned(&input, "AsStringLiteral can only be derived for enums");
            return err.to_compile_error().into();
        }
    };

    // Implement the trait with the generated implementation
    let expanded = quote! {
        impl #impl_generics AsStringLiteral for #name #type_generics #where_clause {
            #as_string_literal_impl
        }
    };

    TokenStream::from(expanded)
}
