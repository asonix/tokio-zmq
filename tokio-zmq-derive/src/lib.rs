extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;

#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use syn::{Attribute, Data, DeriveInput, Fields, Ident, Type};

#[proc_macro_derive(SocketWrapper, attributes(sink, stream, try_from))]
pub fn socket_derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    if !only_has_inner_socket(&input.data) {
        panic!("Expected to derive for struct with inner: Socket");
    }

    let name = input.ident;

    let from_parts = quote! {
        impl From<(zmq::Socket, PollEvented<File<ZmqFile>>)> for #name {
            fn from(tup: (zmq::Socket, PollEvented<File<ZmqFile>>)) -> Self {
                #name {
                    inner: tup.into()
                }
            }
        }
    };

    let as_socket = quote! {
        impl ::prelude::AsSocket for #name {
            fn socket(self) -> Socket {
                self.inner
            }
        }
    };

    let try_from = {
        let socket_type = Ident::from(format!("{}", name).to_uppercase().as_ref());

        let try_from_attr = input.attrs.iter().find(|attr| {
            attr.path
                .segments
                .last()
                .map(|seg| seg.into_value())
                .map(|seg| seg.ident == Ident::new("try_from", seg.ident.span()))
                .unwrap_or(false)
        });

        let try_from_config = if let Some(attr) = try_from_attr {
            attr.tts
                .clone()
                .into_iter()
                .filter_map(|token_tree| match token_tree {
                    TokenTree::Literal(literal) => {
                        Some(Ident::from(format!("{}", literal).trim_matches('"')))
                    }
                    _ => None,
                })
                .next()
        } else {
            Some(Ident::from("SockConfig"))
        };

        let conf = match try_from_config {
            Some(conf) => conf,
            None => panic!("try_from must take the name of the config struct"),
        };

        quote!{
            impl<'a> TryFrom<#conf<'a>> for #name {
                type Error = Error;

                fn try_from(conf: #conf<'a>) -> Result<Self, Self::Error> {
                    Ok(#name {
                        inner: conf.build(zmq::#socket_type)?,
                    })
                }
            }
        }
    };

    let stream = if has_attr(&input.attrs, "stream") {
        quote!{
            impl ::prelude::StreamSocket for #name {}
        }
    } else {
        quote!{}
    };

    let sink = if has_attr(&input.attrs, "sink") {
        quote!{
            impl ::prelude::SinkSocket for #name {}
        }
    } else {
        quote!{}
    };

    let full = quote! {
        #from_parts
        #as_socket
        #stream
        #sink
        #try_from
    };

    full.into()
}

fn has_attr(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| {
        attr.path
            .segments
            .last()
            .map(|seg| seg.into_value())
            .map(|seg| seg.ident == Ident::new(name, seg.ident.span()))
            .unwrap_or(false)
    })
}

fn only_has_inner_socket(input: &Data) -> bool {
    let data_struct = match *input {
        Data::Struct(ref data_struct) => data_struct,
        _ => return false, // TODO: Make this work for enums with sockets in each varient?
    };

    let fields_named = match data_struct.fields {
        Fields::Named(ref fields_named) => fields_named,
        _ => return false, // TODO: Allow other kinds of structs?
    };

    if fields_named.named.len() != 1 {
        return false;
    }

    let field = fields_named.named.first().unwrap().into_value();

    let found = field
        .ident
        .map(|id| id == Ident::new("inner", id.span()))
        .unwrap_or(false);

    if !found {
        return false;
    }

    let type_path = match field.ty {
        Type::Path(ref type_path) => type_path,
        _ => return false,
    };

    type_path
        .path
        .segments
        .last()
        .map(|ps| ps.into_value())
        .map(|ps| ps.ident == Ident::new("Socket", ps.ident.span()))
        .unwrap_or(false)
}
