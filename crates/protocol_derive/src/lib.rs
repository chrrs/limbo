use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, spanned::Spanned, Data, DataStruct, DeriveInput, Field, Fields, Ident, Meta,
};

#[proc_macro_derive(Decodable, attributes(with))]
#[proc_macro_error]
pub fn derive_decodable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let Data::Struct(DataStruct { fields: Fields::Named(mut fields), .. }) = input.data else {
        abort!(input.ident, "Decodable is only derivable for structs");
    };

    let fields = fields.named.iter_mut().map(|field| {
        if let Some(encoder) = find_encoder(field) {
            let name = &field.ident;

            quote_spanned! { encoder.span() =>
                #name: <#encoder>::decode(r).map_err(|e| {
                    crate::DecodingError::Field {
                        name: stringify!(#name),
                        source: Box::new(e),
                    }
                })?,
            }
        } else {
            let name = &field.ident;
            let ty = &field.ty;

            quote_spanned! { ty.span() =>
                #name: <#ty>::decode(r).map_err(|e| {
                    crate::DecodingError::Field {
                        name: stringify!(#name),
                        source: Box::new(e),
                    }
                })?,
            }
        }
    });

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics crate::Decodable for #name #ty_generics #where_clause {
            fn decode(r: &mut impl std::io::Read) -> Result<Self, crate::DecodingError> {
                #[allow(unused_imports)]
                use crate::{Decodable, Decoder};

                Ok(#name {
                    #(#fields)*
                })
            }
        }

    })
}

fn find_encoder(field: &mut Field) -> Option<Ident> {
    field.attrs.iter().find_map(|attr| {
        let Meta::List(list) = &attr.meta else {
            return None;
        };

        if !list.path.is_ident("with") {
            return None;
        }

        let Ok(ident) = list.parse_args::<Ident>() else {
            abort!(list.span(), "with argument should be an identifier");
        };

        Some(ident)
    })
}
