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

#![no_std]
#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

//! Contains helper derive macros for iceoryx2.

extern crate alloc;
extern crate proc_macro;

use alloc::format;
use alloc::vec::Vec;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr, parse_macro_input};

/// Implements the [`iceoryx2_bb_elementary_traits::placement_default::PlacementDefault`] trait when all
/// fields of the struct implement it.
///
/// ```
/// use iceoryx2_bb_derive_macros::PlacementDefault;
/// use iceoryx2_bb_elementary_traits::placement_default::PlacementDefault;
/// use core::alloc::Layout;
/// extern crate alloc;
/// use alloc::alloc::{alloc, dealloc};
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

/// Implements the [`iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend`] trait when all fields of
/// the struct implement it and the struct is annotated with `repr(C)`. A type name can be optionally
/// set with the helper attribute `type_name`.
///
/// ```
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// fn needs_zero_copy_send_type<T: ZeroCopySend>(_: &T) {}
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// #[type_name("MyTypeName")]
/// struct MyZeroCopySendStruct {
///     val1: u64,
///     val2: u64,
/// }
///
/// let x = MyZeroCopySendStruct{
///     val1: 23,
///     val2: 4,
/// };
/// needs_zero_copy_send_type(&x);
/// assert_eq!(unsafe { MyZeroCopySendStruct::type_name() }, "MyTypeName");
///
/// #[repr(C)]
/// #[derive(ZeroCopySend)]
/// #[type_name("GeometricShape")]
/// enum Shape {
///     Point,
///     Circle(f64),
///     Rectangle { width: f64, height: f64 },
/// }
///
/// let shape1 = Shape::Point;
/// let shape2 = Shape::Circle(5.0);
/// let shape3 = Shape::Rectangle { width: 10.0, height: 20.0 };
///
/// needs_zero_copy_send_type(&shape1);
/// needs_zero_copy_send_type(&shape2);
/// needs_zero_copy_send_type(&shape3);
/// assert_eq!(unsafe { Shape::type_name() }, "GeometricShape");
/// ```
#[proc_macro_derive(ZeroCopySend, attributes(type_name))]
pub fn zero_copy_send_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;

    // check for type_name attribute
    let attributes: &Vec<_> = &ast
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("type_name"))
        .collect();
    if attributes.len() > 1 {
        panic!("Too many attributes provided for ZeroCopySend trait.");
    }

    let type_name_impl = match attributes.len() {
        0 => {
            quote! {
                unsafe fn type_name() -> &'static str {
                    core::any::type_name::<Self>()
                }
            }
        }
        _ => {
            let type_name: LitStr = attributes[0]
                .parse_args()
                .expect("Wrong format for ZeroCopySend attribute. Please provide exactly one \"type_name\" in quotation marks.");
            quote! {
                unsafe fn type_name() -> &'static str {
                    #type_name
                }
            }
        }
    };

    // check for repr(C) attribute
    let has_repr_c = &ast.attrs.iter().any(|a| {
        a.path().is_ident("repr")
            && a.parse_args::<syn::Meta>()
                .ok()
                .map(|meta| meta.path().is_ident("C"))
                .unwrap_or(false)
    });
    if !has_repr_c {
        panic!("`#[derive(ZeroCopySend)]` requires the type to be annotated with #[repr(C)]");
    }

    // implement ZeroCopySend
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let zero_copy_send_impl = match ast.data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Named(ref fields_named) => {
                let field_inits = fields_named.named.iter().map(|f| {
                    let field_name = &f.ident;
                    // dummy call to ensure at compile-time that all fields of the struct implement ZeroCopySend
                    quote! {
                        ZeroCopySend::__is_zero_copy_send(&self.#field_name);
                    }
                });

                quote! {
                    fn __is_zero_copy_send(&self) {
                        #(#field_inits)*
                    }

                    #type_name_impl
                }
            }
            Fields::Unnamed(ref fields_unnamed) => {
                let field_inits = fields_unnamed.unnamed.iter().enumerate().map(|(i, _)| {
                    let field_index = syn::Index::from(i);
                    // dummy call to ensure at compile-time that all fields of the struct implement ZeroCopySend
                    quote! {
                        ZeroCopySend::__is_zero_copy_send(&self.#field_index);
                    }
                });

                quote! {
                    fn __is_zero_copy_send(&self) {
                        #(#field_inits)*
                    }

                    #type_name_impl
                }
            }
            Fields::Unit => quote! {
                #type_name_impl
            },
        },
        Data::Enum(ref data_enum) => {
            let variant_checks = data_enum.variants.iter().map(|variant| {
                let variant_name = &variant.ident;

                match &variant.fields {
                    Fields::Named(fields) => {
                        let field_checks = fields.named.iter().map(|f| {
                            let field_name = &f.ident;
                            // dummy call to ensure at compile-time that all fields of the variant implement ZeroCopySend
                            quote! {
                                Self::#variant_name { #field_name, .. } => {
                                    ZeroCopySend::__is_zero_copy_send(#field_name);
                                }
                            }
                        });

                        if fields.named.is_empty() {
                            quote! {
                                Self::#variant_name { .. } => {}
                            }
                        } else {
                            quote! { #(#field_checks)* }
                        }
                    }
                    Fields::Unnamed(fields) => {
                        if fields.unnamed.is_empty() {
                            quote! {
                                Self::#variant_name => {}
                            }
                        } else {
                            let field_names = (0..fields.unnamed.len())
                                .map(|i| {
                                    syn::Ident::new(
                                        &format!("field_{i}"),
                                        proc_macro2::Span::call_site(),
                                    )
                                })
                                .collect::<Vec<_>>();

                            let field_pattern = if field_names.is_empty() {
                                quote! {}
                            } else {
                                quote! { (#(#field_names),*) }
                            };

                            // dummy call to ensure at compile-time that all fields of the variant implement ZeroCopySend
                            let field_checks = field_names.iter().map(|field_name| {
                                quote! {
                                    ZeroCopySend::__is_zero_copy_send(#field_name);
                                }
                            });

                            quote! {
                                Self::#variant_name #field_pattern => {
                                    #(#field_checks)*
                                }
                            }
                        }
                    }
                    Fields::Unit => {
                        quote! {
                            Self::#variant_name => {}
                        }
                    }
                }
            });

            quote! {
                fn __is_zero_copy_send(&self) {
                    match self {
                        #(#variant_checks)*
                    }
                }

                #type_name_impl
            }
        }
        Data::Union(ref data_union) => {
            let field_inits = data_union.fields.named.iter().map(|f| {
                let field_name = &f.ident;
                // dummy call to ensure at compile-time that all fields of the union implement ZeroCopySend
                quote! {
                    ZeroCopySend::__is_zero_copy_send(unsafe { &self.#field_name });
                }
            });

            quote! {
                fn __is_zero_copy_send(&self) {
                    #(#field_inits)*
                }

                #type_name_impl
            }
        }
    };

    let expanded = quote! {
        unsafe impl #impl_generics ZeroCopySend for #struct_name #ty_generics #where_clause {
            #zero_copy_send_impl
        }
    };

    TokenStream::from(expanded)
}

// TODO: documentation
#[proc_macro_derive(AtomicCopy)]
pub fn atomic_copy_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;

    // implement AtomicCopy
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let atomic_copy_impl = match ast.data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Named(ref fields_named) => {
                // implement __for_each_field_with_offset
                let mut field_offsets_and_sizes = Vec::new();
                for field in fields_named.named.iter() {
                    let field_name = &field.ident;
                    let field_type = &field.ty;

                    let block = quote! {
                        let rel_offset = core::mem::offset_of!(#struct_name #ty_generics, #field_name);
                        let abs_offset = base_offset + rel_offset;
                        let size = core::mem::size_of::<#field_type>();

                        // if a field is a struct, its offset and size are not considered,
                        // but the offsets and sizes of its fields are
                        if AtomicCopy::__is_scalar(&self.#field_name) {
                            callback(abs_offset, size);
                        }
                        else {
                            AtomicCopy::__for_each_field_with_offset(&self.#field_name, abs_offset, callback);
                        }
                    };
                    field_offsets_and_sizes.push(block);
                }

                quote! {
                    fn __for_each_field_with_offset<F: FnMut(usize, usize)>(&self, base_offset: usize, callback: &mut F) {
                        #(#field_offsets_and_sizes)*
                    }
                }
            }
            Fields::Unnamed(ref fields_unnamed) => {
                // implement __for_each_field_with_offset
                let mut field_offsets_and_sizes = Vec::new();
                for (i, field) in fields_unnamed.unnamed.iter().enumerate() {
                    let field_index = syn::Index::from(i);
                    let field_type = &field.ty;

                    let block = quote! {
                        let rel_offset = core::mem::offset_of!(#struct_name #ty_generics, #field_index);
                        let abs_offset = base_offset + rel_offset;
                        let size = core::mem::size_of::<#field_type>();

                        // if a field is a struct, its offset and size are not considered,
                        // but the offsets and sizes of its fields are
                        if AtomicCopy::__is_scalar(&self.#field_index) {
                            callback(abs_offset, size);
                        }
                        else {
                            AtomicCopy::__for_each_field_with_offset(&self.#field_index, abs_offset, callback);
                        }
                    };
                    field_offsets_and_sizes.push(block);
                }

                quote! {
                    fn __for_each_field_with_offset<F: FnMut(usize, usize)>(&self, base_offset: usize, callback: &mut F) {
                        #(#field_offsets_and_sizes)*
                    }
                }
            }
            Fields::Unit => quote! {},
        },
        Data::Enum(_) => {
            // Can be implemented once core::mem::offset_of! is stable for enums.
            panic!("`#[derive(AtomicCopy)]` is not implemented for enums, only for structs.");
        }
        Data::Union(_) => {
            panic!("`#[derive(AtomicCopy)]` is not implemented for unions, only for structs.");
        }
    };

    let expanded = quote! {
        unsafe impl #impl_generics AtomicCopy for #struct_name #ty_generics #where_clause {
            #atomic_copy_impl
        }
    };

    TokenStream::from(expanded)
}

/// Implements the [`iceoryx2_bb_elementary_traits::zeroable::Zeroable`] trait
/// when all fields of the struct also implement it. Rejects enums, unions, and
/// structs whose fields violate the `Zeroable` invariant at compile time.
///
/// ```
/// use iceoryx2_bb_derive_macros::Zeroable;
/// use iceoryx2_bb_elementary_traits::zeroable::Zeroable;
///
/// #[derive(Zeroable)]
/// struct MyStruct {
///     val1: u64,
///     val2: [u8; 16],
/// }
///
/// let v = MyStruct::new_zeroed();
/// assert_eq!(v.val1, 0);
/// ```
///
#[proc_macro_derive(Zeroable)]
pub fn zeroable_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let impl_body = match &ast.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => {
                let calls = fields.named.iter().map(|field| {
                    let name = &field.ident;
                    quote! { Zeroable::__is_zeroable(&self.#name); }
                });
                quote! {
                    fn __is_zeroable(&self) {
                        #(#calls)*
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let calls = fields.unnamed.iter().enumerate().map(|(i, _)| {
                    let idx = syn::Index::from(i);
                    quote! { Zeroable::__is_zeroable(&self.#idx); }
                });
                quote! {
                    fn __is_zeroable(&self) {
                        #(#calls)*
                    }
                }
            }
            Fields::Unit => quote! {},
        },
        _ => panic!("`#[derive(Zeroable)]` can only be used on structs"),
    };

    let expanded = quote! {
        unsafe impl #impl_generics Zeroable for #name #ty_generics #where_clause {
            #impl_body
        }
    };

    TokenStream::from(expanded)
}

/// Implements the [`iceoryx2_bb_elementary_traits::plain_old_data_without_padding::PlainOldDataWithoutPadding`]
/// trait when the struct is annotated with `#[repr(C)]`, has no padding
/// between or after its fields, and every field also implements the trait.
/// Rejects enums, unions, layouts with padding, and supertrait violations
/// (`Copy`, `Zeroable`, `ZeroCopySend`, `'static`) at compile time.
///
/// ```
/// use iceoryx2_bb_derive_macros::{PlainOldDataWithoutPadding, ZeroCopySend, Zeroable};
/// use iceoryx2_bb_elementary_traits::plain_old_data_without_padding::PlainOldDataWithoutPadding;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
/// use iceoryx2_bb_elementary_traits::zeroable::Zeroable;
///
/// #[repr(C)]
/// #[derive(Copy, Clone, Zeroable, ZeroCopySend, PlainOldDataWithoutPadding)]
/// struct MyPodStruct {
///     val1: u64,
///     val2: [u8; 16],
/// }
///
/// fn needs_pod<T: PlainOldDataWithoutPadding>(_: &T) {}
/// needs_pod(&MyPodStruct { val1: 0, val2: [0; 16] });
/// ```
///
#[proc_macro_derive(PlainOldDataWithoutPadding)]
pub fn plain_old_data_without_padding_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    // reject enum/union, collect field accessors + field types
    let (field_accessors, field_types): (Vec<_>, Vec<_>) = match &ast.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .map(|field| {
                    let ident = &field.ident;
                    let ty = &field.ty;
                    (quote! { #ident }, ty)
                })
                .unzip(),
            Fields::Unnamed(fields) => fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, field)| {
                    let idx = syn::Index::from(i);
                    let ty = &field.ty;
                    (quote! { #idx }, ty)
                })
                .unzip(),
            Fields::Unit => (Vec::new(), Vec::new()),
        },
        _ => panic!("`#[derive(PlainOldDataWithoutPadding)]` can only be used on structs"),
    };

    // verify #[repr(C)]
    let has_repr_c = ast.attrs.iter().any(|a| {
        a.path().is_ident("repr")
            && a.parse_args::<syn::Meta>()
                .ok()
                .map(|m| m.path().is_ident("C"))
                .unwrap_or(false)
    });
    if !has_repr_c {
        panic!(
            "`#[derive(PlainOldDataWithoutPadding)]` requires the type to be annotated with #[repr(C)]"
        );
    }

    // padding check via const assert
    let size_check = if field_types.is_empty() {
        quote! {
            assert!(
                ::core::mem::size_of::<#name #ty_generics>() == 0,
                "PlainOldDataWithoutPadding: unit struct must have size 0"
            );
        }
    } else {
        quote! {
            assert!(
                ::core::mem::size_of::<#name #ty_generics>()
                    == 0usize #( + ::core::mem::size_of::<#field_types>() )*,
                "PlainOldDataWithoutPadding: struct has padding bytes"
            );
        }
    };

    // dummy-call body to verify each field is PoD
    let call_body = field_accessors.iter().map(|acc| {
        quote! { PlainOldDataWithoutPadding::__is_pod(&self.#acc); }
    });

    let expanded = quote! {
        const _: () = { #size_check };

        unsafe impl #impl_generics PlainOldDataWithoutPadding
            for #name #ty_generics #where_clause
        {
            fn __is_pod(&self) {
                #(#call_body)*
            }
        }
    };

    TokenStream::from(expanded)
}

#[cfg(doctest)]
mod zero_copy_send_compile_tests;

#[cfg(doctest)]
mod zeroable_compile_tests;

#[cfg(doctest)]
mod plain_old_data_without_padding_compile_tests;

#[cfg(doctest)]
mod atomic_copy_compile_tests;
