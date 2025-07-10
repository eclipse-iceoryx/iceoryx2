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

use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, punctuated::Punctuated, Data, DeriveInput, Expr, ExprLit, Fields, Lit,
    LitStr, Meta, Token,
};

#[proc_macro_attribute]
pub fn iceoryx2_ffi(args: TokenStream, input: TokenStream) -> TokenStream {
    let Args { rust_type: my_type } = parse_attribute_args(args);

    // Parse the input tokens into a syntax tree
    let my_struct = parse_macro_input!(input as DeriveInput);

    let mut has_repr_c = false;
    for attr in &my_struct.attrs {
        if attr.path().is_ident("repr") {
            let nested = attr
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .expect("'repr' should have at least one argument");
            for meta in nested {
                match meta {
                    // #[repr(C)]
                    Meta::Path(path) if path.is_ident("C") => {
                        has_repr_c = true;
                    }
                    _ => (),
                }
            }
        }
    }

    if !has_repr_c {
        panic!(
            "The 'repr(C)' attribute is missing from '{}'!",
            &my_struct.ident.to_string()
        );
    }

    // Get the name of the struct we are generating code
    let struct_name = &my_struct.ident;
    let stripped_struct_name = struct_name
        .to_string()
        .strip_prefix("iox2_")
        .expect("The struct must have an 'iox2_' prefix")
        .strip_suffix("_t")
        .expect("The struct must have a '_t' suffix")
        .to_string();

    // Define all the names we need
    let struct_storage_name = format_ident!("iox2_{}_storage_t", stripped_struct_name);
    let _struct_h_t_name = format_ident!("iox2_{}_h_t", stripped_struct_name);
    let struct_h_name = format_ident!("iox2_{}_h", stripped_struct_name);
    let _struct_h_ref_name = format_ident!("iox2_{}_h_ref", stripped_struct_name);

    // NOTE: cbindgen does not play well with adding new structs or fields to existing structs;
    // this code is kept for reference

    // // Add the additional fields to the struct
    // match &mut my_struct.data {
    //     syn::Data::Struct(ref mut struct_data) => match &mut struct_data.fields {
    //         syn::Fields::Named(fields) => {
    //             fields.named.push(
    //                 syn::Field::parse_named
    //                 .parse2(quote! { value: #struct_storage_name })
    //                 .unwrap(),
    //             );
    //             fields.named.push(
    //                 syn::Field::parse_named
    //                 .parse2(quote! { deleter: fn(*mut #struct_name) })
    //                 .unwrap(),
    //             );
    //         }
    //         _ => (),
    //     },
    //     _ => panic!("#### `iceoryx_ffi` has to be used with structs "),
    // };

    let expanded = quote! {
        impl #struct_storage_name {
            const fn assert_storage_layout() {
                static_assert_ge::<
                { ::core::mem::align_of::<#struct_storage_name>() },
                { ::core::mem::align_of::<Option<#my_type>>() },
                >();
                static_assert_ge::<
                { ::core::mem::size_of::<#struct_storage_name>() },
                { ::core::mem::size_of::<Option<#my_type>>() },
                >();
            }

            fn init(&mut self, value: #my_type) {
                #struct_storage_name::assert_storage_layout();

                unsafe { &mut *(self as *mut Self).cast::<::core::mem::MaybeUninit<Option<#my_type>>>() }
                .write(Some(value));
            }

            pub(super) unsafe fn as_option_mut(&mut self) -> &mut Option<#my_type> {
                &mut *(self as *mut Self).cast::<Option<#my_type>>()
            }

            pub(super) unsafe fn as_option_ref(&self) -> &Option<#my_type> {
                &*(self as *const Self).cast::<Option<#my_type>>()
            }

            pub(super) unsafe fn as_mut(&mut self) -> &mut #my_type {
                self.as_option_mut().as_mut().unwrap()
            }

            pub(super) unsafe fn as_ref(&self) -> &#my_type {
                self.as_option_ref().as_ref().unwrap()
            }
        }

        // this is the struct which is annotated with '#[iceoryx2_ffi(Type)]'
        #my_struct

        impl #struct_name {
            pub(super) fn as_handle(&mut self) -> #struct_h_name {
                self as *mut _ as _
            }

            pub(super) fn take(&mut self) -> Option<#my_type> {
                unsafe { self.value.as_option_mut().take() }
            }

            pub(super) fn set(&mut self, value: #my_type) {
                unsafe { *self.value.as_option_mut() = Some(value) }
            }

            pub(super) fn alloc() -> *mut #struct_name {
                unsafe { ::alloc::alloc::alloc(::core::alloc::Layout::new::<#struct_name>()) as _ }
            }

            pub(super) fn dealloc(storage: *mut #struct_name) {
                unsafe {
                    ::alloc::alloc::dealloc(storage as _, ::core::alloc::Layout::new::<#struct_name>())
                }
            }
        }

        #[cfg(test)]
        mod test_generated {
            use super::*;

            #[test]
            fn assert_storage_size() {
                // all const functions; if it compiles, the storage size is sufficient
                const _STORAGE_LAYOUT_CHECK: () = #struct_storage_name::assert_storage_layout();
            }
        }
    };

    // eprintln!("#### DEBUG\n{}", expanded);
    TokenStream::from(expanded)
}

struct Args {
    rust_type: syn::Type,
}

fn parse_attribute_args(args: TokenStream) -> Args {
    let args = proc_macro2::TokenStream::from(args);

    let args = args.into_iter().collect::<Vec<_>>();

    let attribute_format = "Format must be '#[iceoryx2_ffi(Type)]'";
    if args.len() != 1 {
        panic!("Invalid attribute definition! {attribute_format}");
    }

    let rust_type = match &args[0] {
        TokenTree::Ident(my_type) => LitStr::new(&my_type.to_string(), my_type.span())
            .parse::<syn::Type>()
            .expect("Valid type"),
        _ => panic!("Invalid type argument! {attribute_format}"),
    };

    // NOTE: this code is kept for reference if more arguments are added to the attribute

    // match (&args[1], &args[3], &args[5], &args[7]) {
    //     (TokenTree::Punct(a), TokenTree::Punct(b), TokenTree::Punct(c), TokenTree::Punct(d))
    //         if a.as_char() == ','
    //             && b.as_char() == '='
    //             && c.as_char() == ','
    //             && d.as_char() == '=' =>
    //     {
    //         ()
    //     }
    //     _ => panic!("Invalid format! {}", attribute_format),
    // }
    //
    // let size = match (&args[2], &args[4]) {
    //     (TokenTree::Ident(key), TokenTree::Literal(value)) if key.to_string() == "size" => {
    //         <proc_macro2::Literal as Into<LitInt>>::into(value.clone())
    //     }
    //     _ => panic!("Invalid 'size' argument! {}", attribute_format),
    // };
    //
    // let alignment = match (&args[6], &args[8]) {
    //     (TokenTree::Ident(key), TokenTree::Literal(value)) if key.to_string() == "alignment" => {
    //         <proc_macro2::Literal as Into<LitInt>>::into(value.clone())
    //     }
    //     _ => panic!("Invalid 'alignment' argument! {}", attribute_format),
    // };

    Args { rust_type }
}

/// Implements the [`iceoryx2_bb_elementary_traits::AsCStr`] trait for enums to provide a string representation of each enum variant.
///
/// The string representation can be customized using the `CStr` attribute, otherwise it will
/// convert the variant name to lowercase and replace underscores with spaces.
///
/// # Example
/// ```
/// use iceoryx2_ffi_macros::CStrRepr;
/// use iceoryx2_bb_elementary_traits::AsCStr;
///
/// #[derive(CStrRepr)]
/// enum MyEnum {
///     #[CStr = "custom variant one"]
///     VariantOne,
///     VariantTwo,
/// }
///
/// let v1 = MyEnum::VariantOne;
/// assert_eq!(v1.as_const_cstr(), c"custom variant one");
///
/// let v2 = MyEnum::VariantTwo;
/// assert_eq!(v2.as_const_cstr(), c"variant two");
/// ```
#[proc_macro_derive(CStrRepr, attributes(CStr))]
pub fn string_literal_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let as_cstr_impl = match input.data {
        Data::Enum(ref data_enum) => {
            let enum_to_string_mapping = data_enum.variants.iter().map(|variant| {
                let null_terminated = |s: &str| {
                    quote! {
                        // This code appends the null termination which cannot be confirmed at compile time,
                        // thus the code is ensured safe.
                        unsafe { ::core::ffi::CStr::from_bytes_with_nul_unchecked(concat!(#s, "\0").as_bytes()) }
                    }
                };

                let enum_name = &variant.ident;
                let cstr_literal = variant
                    .attrs
                    .iter()
                    .find(|attr| attr.path().is_ident("CStr"))
                    .and_then(|attr| match attr.meta.require_name_value() {
                        Ok(meta) => match &meta.value {
                            Expr::Lit(ExprLit { lit: Lit::Str(lit), .. }) => Some(null_terminated(&lit.value())),
                            _ => None,
                        },
                        _ => None,
                    })
                    .unwrap_or_else(|| {
                        // No explicity `CStr` is provided.
                        // Convert variant name from 'UpperCamelCase' to 'lowercase with spaces'.
                        let enum_string = enum_name.to_string()
                            .char_indices()
                            .fold(String::new(), |mut acc, (i, c)| {
                                if i > 0 && c.is_uppercase() {
                                    acc.push(' ');
                                }
                                acc.push(c.to_ascii_lowercase());
                                acc
                            });
                        null_terminated(&enum_string)
                    });

                let pattern = match &variant.fields {
                    Fields::Unit => quote!(Self::#enum_name),
                    Fields::Unnamed(_) => quote!(Self::#enum_name(..)),
                    Fields::Named(_) => quote!(Self::#enum_name{..}),
                };
                quote! { #pattern => #cstr_literal }
            });

            quote! {
                fn as_const_cstr(&self) -> &'static ::core::ffi::CStr {
                    match self {
                        #(#enum_to_string_mapping,)*
                    }
                }
            }
        }
        _ => {
            let err = syn::Error::new_spanned(&input, "AsCStrRepr can only be derived for enums");
            return err.to_compile_error().into();
        }
    };

    TokenStream::from(quote! {
        impl #impl_generics AsCStr for #name #type_generics #where_clause {
            #as_cstr_impl
        }
    })
}
