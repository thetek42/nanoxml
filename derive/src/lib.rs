use proc_macro::TokenStream;

#[cfg(feature = "ser")]
#[proc_macro_derive(SerXml, attributes(attr, text))]
pub fn derive_serxml(input: TokenStream) -> TokenStream {
    use quote::quote;
    use syn::{Data, DataStruct, DeriveInput, Fields, parse_macro_input};

    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => &fields.named,
        _ => panic!("SerXml can only be derived for structs with named fields"),
    };

    let mut regular_fields = Vec::new();
    let mut attr_fields = Vec::new();
    let mut text_field = None;
    for field in fields {
        let field_name = &field.ident;
        let is_attr = field.attrs.iter().any(|attr| attr.path().is_ident("attr"));
        let is_text = field.attrs.iter().any(|attr| attr.path().is_ident("text"));
        match (is_attr, is_text) {
            (true, true) => panic!("#[attr] and #[text] are incompatible"),
            (true, false) => attr_fields.push(field_name),
            (false, true) => {
                if text_field.is_some() {
                    panic!("only one #[text] field is allowed");
                }
                if !regular_fields.is_empty() {
                    panic!("#[text] is incompatible with regular fields");
                }
                text_field = Some(field_name);
            }
            (false, false) => {
                if text_field.is_some() {
                    panic!("#[text] is incompatible with regular fields");
                }
                regular_fields.push(field_name);
            }
        }
    }

    let ser_body = match text_field {
        Some(text_field) => quote! {
            ::nanoxml::derive::SerXmlNoAttrs::ser_as_body(&self.#text_field, xml)
        },
        None => quote! {
            #(::nanoxml::derive::SerXml::ser(&self.#regular_fields, xml, stringify!(#regular_fields))?;)*
            Ok(())
        },
    };

    let ser_attrs = quote! {
        #(::nanoxml::derive::SerXmlAsAttr::ser_as_attr(&self.#attr_fields, xml, stringify!(#attr_fields))?;)*
        Ok(())
    };

    let serxml_impl = quote! {
        impl ::nanoxml::derive::SerXml for #name {
            fn ser_body<W: ::core::fmt::Write>(&self, xml: &mut ::nanoxml::ser::XmlBuilder<'_, W>) -> ::core::fmt::Result {
                #ser_body
            }

            fn ser_attrs<W: ::core::fmt::Write>(&self, xml: &mut ::nanoxml::ser::XmlBuilder<'_, W>) -> ::core::fmt::Result {
                #ser_attrs
            }
        }
    };

    let no_attr_impl = match attr_fields.len() {
        0 => quote! { impl ::nanoxml::derive::SerXmlNoAttrs for #name {} },
        _ => quote! {},
    };

    let top_level_impl = quote! {
        impl ::nanoxml::derive::SerXmlTopLevel for #name {
            const TAG_NAME: &'static str = stringify!(#name);
        }
    };

    let full_impl = quote! {
        #serxml_impl
        #no_attr_impl
        #top_level_impl
    };

    full_impl.into()
}
