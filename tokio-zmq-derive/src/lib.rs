extern crate proc_macro;
extern crate syn;

#[macro_use]
extern crate quote;
#[macro_use]
extern crate synstructure;

decl_derive!([SocketWrapper, attributes(stream, sink, try_from)] => socket_derive);

fn socket_derive(s: synstructure::Structure) -> quote::Tokens {
    let socket_binding = s.variants().iter().find(|v| {
        v.bindings().iter().any(|b| {
            if let Some(ref ident) = b.ast().ident {
                if ident == "inner" {
                    return true;
                }
            }

            false
        })
    });

    if let Some(sb) = socket_binding {
        let as_socket = s.bound_impl(
            "::prelude::AsSocket",
            quote! {
                fn socket(self) -> Socket {
                    self.inner
                }
            },
        );

        let name = sb.ast().ident;
        let try_from = build_try_from(&s, name);

        let stream = if has_attr(&s, "stream") {
            s.bound_impl("::prelude::StreamSocket", quote!{})
        } else {
            quote!{}
        };

        let sink = if has_attr(&s, "sink") {
            s.bound_impl("::prelude::SinkSocket", quote!{})
        } else {
            quote!{}
        };

        quote! {
            #as_socket
            #stream
            #sink
            #try_from
        }
    } else {
        panic!("Could not find socket");
    }
}

fn has_attr(s: &synstructure::Structure, attr: &str) -> bool {
    s.ast().attrs.iter().any(|a| is_attr(a, attr))
}

fn is_attr(a: &syn::Attribute, attr: &str) -> bool {
    a.name() == attr
}

fn build_try_from(s: &synstructure::Structure, name: &syn::Ident) -> quote::Tokens {
    let socket_type = syn::Ident::new(format!("{}", name).to_uppercase());

    let try_from_attr = s.ast().attrs.iter().find(|a| is_attr(a, "try_from"));

    let mut try_from_conf = None;

    if let Some(ref try_from) = try_from_attr {
        if let syn::MetaItem::NameValue(_, syn::Lit::Str(ref name, _)) = try_from.value {
            let name: String = name.to_owned();
            try_from_conf = Some(syn::Ident::new(name));
        } else {
            panic!("try_from must take the name of the config struct");
        }
    }

    let conf = try_from_conf.unwrap_or(syn::Ident::new("SockConfig"));

    quote! {
        impl<'a> TryFrom<#conf<'a>> for #name {
            type Error = Error;

            fn try_from(conf: #conf<'a>) -> Result<Self, Self::Error> {
                Ok(#name {
                    inner: conf.build(zmq::#socket_type)?,
                })
            }
        }
    }
}
