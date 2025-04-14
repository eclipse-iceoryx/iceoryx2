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

#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

//! Contains helper derive macros for iceoryx2.

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Expr, Fields};

/// Implements the [`iceoryx2_bb_elementary::placement_default::PlacementDefault`] trait when all
/// fields of the struct implement it.
///
/// ```
/// use iceoryx2_bb_derive_macros::PlacementDefault;
/// use iceoryx2_bb_elementary::placement_default::PlacementDefault;
/// use core::alloc::Layout;
/// use std::alloc::{alloc, dealloc};
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

/// Implements the [`iceoryx2_bb_elementary::zero_copy_send::ZeroCopySend`] trait when all fields of
/// the struct implement it. A type name can be optionally set with the helper attribute `type_name`.
///
/// ```
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary::zero_copy_send::ZeroCopySend;
///
/// #[derive(ZeroCopySend)]
/// #[type_name(MyTypeName)]
/// struct MyZeroCopySendStruct {
///     val1: u64,
///     val2: u64,
/// }
///
/// fn needs_zero_copy_send_type<T: ZeroCopySend>(_: &T) {}
///
/// let x = MyZeroCopySendStruct{
///     val1: 23,
///     val2: 4,
/// };
/// needs_zero_copy_send_type(&x);
/// assert_eq!(unsafe { MyZeroCopySendStruct::type_name() }, "MyTypeName");
/// ```
#[proc_macro_derive(ZeroCopySend, attributes(type_name))]
pub fn zero_copy_send_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    // check attribute
    let attributes: &Vec<_> = &ast
        .attrs
        .iter()
        .filter(|a| a.path().segments.len() == 1 && a.path().segments[0].ident == "type_name")
        .collect();
    if attributes.len() > 1 {
        panic!("Too many attributes provided for ZeroCopySend trait.");
    }
    let mut attribute: Option<Expr> = None;
    if attributes.len() == 1 {
        attribute = Some(attributes[0].parse_args().expect(
            "Wrong format for ZeroCopySend attribute. Please provide exactly one \'type_name\'.",
        ));
    }

    let struct_name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let type_name_impl = match ast.data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Named(ref fields_named) => {
                let field_inits = fields_named.named.iter().map(|f| {
                    let field_name = &f.ident;
                    // dummy call to ensure at compile-time that all fields of the struct implement ZeroCopySend
                    quote! {
                        iceoryx2_bb_elementary::zero_copy_send::ZeroCopySend::__is_zero_copy_send(&self.#field_name);
                    }
                });

                match attribute {
                    None => {
                        quote! {
                            fn __is_zero_copy_send(&self) {
                                #(#field_inits)*
                            }
                            unsafe fn type_name() -> &'static str {
                                core::any::type_name::<Self>()
                            }
                        }
                    }
                    Some(_) => {
                        quote! {
                            fn __is_zero_copy_send(&self) {
                                #(#field_inits)*
                            }
                            unsafe fn type_name() -> &'static str {
                                stringify!(#attribute)
                            }
                        }
                    }
                }
            }
            Fields::Unnamed(ref fields_unnamed) => {
                let field_inits = fields_unnamed.unnamed.iter().enumerate().map(|(i, _)| {
                    let field_index = syn::Index::from(i);
                    // dummy call to ensure at compile-time that all fields of the struct implement ZeroCopySend
                    quote! {
                        iceoryx2_bb_elementary::zero_copy_send::ZeroCopySend::__is_zero_copy_send(&self.#field_index);
                    }
                });

                match attribute {
                    None => {
                        quote! {
                            fn __is_zero_copy_send(&self) {
                                #(#field_inits)*
                            }
                            unsafe fn type_name() -> &'static str {
                                core::any::type_name::<Self>()
                            }
                        }
                    }
                    Some(_) => {
                        quote! {
                            fn __is_zero_copy_send(&self) {
                                #(#field_inits)*
                            }
                            unsafe fn type_name() -> &'static str {
                                stringify!(#attribute)
                            }
                        }
                    }
                }
            }
            Fields::Unit => match attribute {
                None => {
                    quote! {
                        unsafe fn type_name() -> &'static str {
                            core::any::type_name::<Self>()
                        }
                    }
                }
                Some(_) => {
                    quote! {
                        unsafe fn type_name() -> &'static str {
                            stringify!(#attribute)
                        }
                    }
                }
            },
        },
        _ => {
            return quote! {compile_error!("ZeroCopySend can only be implemented for structs");}
                .into();
        }
    };

    let expanded = quote! {
        // TODO: repr(C)
        unsafe impl #impl_generics iceoryx2_bb_elementary::zero_copy_send::ZeroCopySend for #struct_name #ty_generics #where_clause {
            #type_name_impl
        }
    };

    TokenStream::from(expanded)
}
// TODO: check other containers + lock-free

#[cfg(doctest)]
mod zero_copy_send_compile_tests;
