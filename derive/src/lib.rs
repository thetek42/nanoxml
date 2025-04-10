use proc_macro::TokenStream;

#[cfg(feature = "ser")]
#[proc_macro_derive(SerXml, attributes(attr, rename, text))]
pub fn derive_serxml(input: TokenStream) -> TokenStream {
    use quote::quote;
    use syn::{Data, DataStruct, DeriveInput, Expr, Fields, Lit, parse_macro_input};

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
        let field_name = field.ident.as_ref().unwrap();
        let is_attr = field.attrs.iter().any(|attr| attr.path().is_ident("attr"));
        let is_text = field.attrs.iter().any(|attr| attr.path().is_ident("text"));
        let rename = field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("rename"))
            .next()
            .and_then(|attr| attr.meta.require_name_value().ok())
            .and_then(|attr| match &attr.value {
                Expr::Lit(lit) => Some(lit),
                _ => None,
            })
            .and_then(|lit| match &lit.lit {
                Lit::Str(lit) => Some(lit),
                _ => None,
            })
            .map(|lit| lit.value());
        let is_rename = rename.is_some();
        let rename = rename.unwrap_or(field_name.to_string());
        match (is_attr, is_text) {
            (true, true) => panic!("#[attr] and #[text] are incompatible"),
            (true, false) => attr_fields.push((field_name, rename)),
            (false, true) if text_field.is_some() => panic!("only one #[text] field is allowed"),
            (false, true) if !regular_fields.is_empty() => {
                panic!("#[text] is incompatible with regular fields")
            }
            (false, true) if is_rename => panic!("#[text] and #[rename] are incompatible"),
            (false, true) => text_field = Some(field_name),
            (false, false) if text_field.is_some() => {
                panic!("#[text] is incompatible with regular fields")
            }
            (false, false) => regular_fields.push((field_name, rename)),
        }
    }

    let ser_body = match text_field {
        Some(text_field) => vec![quote! { ::nanoxml::derive::SerXmlNoAttrs::ser_as_body(&self.#text_field, xml)?; }],
        None => regular_fields
            .iter()
            .map(|(field, rename)| quote! { ::nanoxml::derive::SerXml::ser(&self.#field, xml, #rename)?; })
            .collect(),
    };

    let ser_attrs: Vec<_> = attr_fields
        .iter()
        .map(|(field, rename)| quote! { ::nanoxml::derive::SerXmlAsAttr::ser_as_attr(&self.#field, xml, #rename)?; })
        .collect();

    let serxml_impl = quote! {
        impl ::nanoxml::derive::SerXml for #name {
            fn ser_body<W: ::core::fmt::Write>(&self, xml: &mut ::nanoxml::ser::XmlBuilder<'_, W>) -> ::core::fmt::Result {
                #(#ser_body)*
                Ok(())
            }

            fn ser_attrs<W: ::core::fmt::Write>(&self, xml: &mut ::nanoxml::ser::XmlBuilder<'_, W>) -> ::core::fmt::Result {
                #(#ser_attrs)*
                Ok(())
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
