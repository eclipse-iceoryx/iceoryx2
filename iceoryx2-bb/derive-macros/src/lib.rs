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
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

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
