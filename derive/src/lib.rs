use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DataStruct, DeriveInput, Fields};

#[proc_macro_derive(Packet)]
pub fn packet(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    match data {
        syn::Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => {
            let read = fields.named.iter().map(|field| {
                let ident = field.ident.clone();
                let ty = field.ty.clone();
                quote! {
                    #ident: #ty::read_from(buffer)?,
                }
            });

            let write = fields.named.iter().map(|field| {
                let ident = field.ident.clone();
                quote! {
                    self.#ident.write_to(buffer)?;
                }
            });

            (quote! {
                impl crate::protocol::Readable for #ident {
                    fn read_from(
                        buffer: &mut std::io::Cursor<&[u8]>,
                    ) -> Result<Self, crate::protocol::ProtocolError> {
                        Ok(Self {
                            #(#read)*
                        })
                    }
                }

                impl crate::protocol::Writable for #ident {
                    fn write_to(&self, buffer: &mut Vec<u8>) -> Result<(), crate::protocol::ProtocolError> {
                        #(#write)*
                        Ok(())
                    }
                }
            })
            .into()
        }
        _ => quote! { compile_error!("can only derive for structs with named fields"); }.into(),
    }
}
