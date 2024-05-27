extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(PlacementDefault)]
pub fn placement_default_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let place_new_impl = match input.data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Named(ref fields_named) => {
                let field_inits = fields_named.named.iter().map(|f| {
                    let name = &f.ident;
                    quote! {
                        PlacementDefault::placement_default(&mut (*ptr).#name);
                    }
                });

                quote! {
                    unsafe fn placement_default(ptr: *mut #name) {
                        #(#field_inits)*
                    }
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let expanded = quote! {
        impl iceoryx2_bb_elementary::placement_new::PlacementDefault for #name {
            #place_new_impl
        }
    };

    TokenStream::from(expanded)
}
