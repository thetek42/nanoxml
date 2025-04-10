#![allow(unused_imports)]

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Attribute, Data, DataStruct, DeriveInput, Expr, Field, Fields, Ident, Lit, Variant};

#[cfg(feature = "ser")]
#[proc_macro_derive(SerXml, attributes(attr, rename, text))]
pub fn derive_serxml(input: TokenStream) -> TokenStream {
    use syn::DataEnum;

    let input = parse_macro_input!(input as DeriveInput);
    let rename = get_rename_attr(&input.attrs).unwrap_or(input.ident.to_string());
    match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => derive_serxml_struct(&input.ident, &rename, &fields.named),
        Data::Enum(DataEnum { variants, .. }) => {
            derive_serxml_enum(&input.ident, &rename, &variants)
        }
        _ => panic!("SerXml can only be derived for structs with named fields or enums"),
    }
}

#[cfg(feature = "ser")]
fn derive_serxml_struct(
    name: &Ident,
    rename: &str,
    fields: &Punctuated<Field, Comma>,
) -> TokenStream {
    let xml_fields = get_xml_fields(fields);

    let ser_body = match xml_fields.text {
        Some(text_field) => {
            vec![
                quote! { ::nanoxml::derive::ser::SerXmlNoAttrs::ser_as_body(&self.#text_field, xml)?; },
            ]
        }
        None => xml_fields
            .regular
            .iter()
            .map(|field| {
                let RenamedField {
                    field_name,
                    renamed,
                } = field;
                quote! { ::nanoxml::derive::ser::SerXml::ser_xml(&self.#field_name, xml, #renamed)?; }
            })
            .collect(),
    };

    let ser_attrs: Vec<_> = xml_fields.attrs
        .iter()
        .map(|field| {
            let RenamedField {
                field_name,
                renamed,
            } = field;
            quote! { ::nanoxml::derive::ser::SerXmlAsAttr::ser_as_attr(&self.#field_name, xml, #renamed)?; }
        })
        .collect();

    let serxml_impl = quote! {
        impl ::nanoxml::derive::ser::SerXml for #name {
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

    let no_attr_impl = match xml_fields.attrs.len() {
        0 => quote! { impl ::nanoxml::derive::ser::SerXmlNoAttrs for #name {} },
        _ => quote! {},
    };

    let top_level_impl = quote! {
        impl ::nanoxml::derive::ser::SerXmlTopLevel for #name {
            const TAG_NAME: &'static str = #rename;
        }
    };

    let full_impl = quote! {
        #serxml_impl
        #no_attr_impl
        #top_level_impl
    };

    full_impl.into()
}

#[cfg(feature = "ser")]
fn derive_serxml_enum(
    name: &Ident,
    rename: &str,
    variants: &Punctuated<Variant, Comma>,
) -> TokenStream {
    let variants = get_xml_variants(variants);

    let cases: Vec<_> = variants
        .iter()
        .map(|variant| {
            let RenamedField {
                field_name: variant_name,
                renamed,
            } = variant;
            quote! { Self::#variant_name => xml.text(#renamed), }
        })
        .collect();

    let serxml_impl = quote! {
        impl ::nanoxml::derive::ser::SerXml for #name {
            fn ser_body<W: ::core::fmt::Write>(&self, xml: &mut ::nanoxml::ser::XmlBuilder<'_, W>) -> ::core::fmt::Result {
                match self {
                    #(#cases)*
                }
            }

            fn ser_attrs<W: ::core::fmt::Write>(&self, xml: &mut ::nanoxml::ser::XmlBuilder<'_, W>) -> ::core::fmt::Result {
                Ok(())
            }
        }
    };

    let no_attr_impl = quote! { impl ::nanoxml::derive::ser::SerXmlNoAttrs for #name {} };

    let as_attr_impl = quote! { impl ::nanoxml::derive::ser::SerXmlAsAttr for #name {} };

    let top_level_impl = quote! {
        impl ::nanoxml::derive::ser::SerXmlTopLevel for #name {
            const TAG_NAME: &'static str = #rename;
        }
    };

    let full_impl = quote! {
        #serxml_impl
        #no_attr_impl
        #as_attr_impl
        #top_level_impl
    };

    full_impl.into()
}

fn get_rename_attr(attrs: &[Attribute]) -> Option<String> {
    attrs
        .iter()
        .find(|attr| attr.path().is_ident("rename"))
        .and_then(|attr| attr.meta.require_name_value().ok())
        .and_then(|attr| match &attr.value {
            Expr::Lit(lit) => Some(lit),
            _ => None,
        })
        .and_then(|lit| match &lit.lit {
            Lit::Str(lit) => Some(lit),
            _ => None,
        })
        .map(|lit| lit.value())
}

struct XmlFields<'a> {
    regular: Vec<RenamedField<'a>>,
    attrs: Vec<RenamedField<'a>>,
    text: Option<&'a Ident>,
}

struct RenamedField<'a> {
    field_name: &'a Ident,
    renamed: String,
}

fn get_xml_fields(fields: &Punctuated<Field, Comma>) -> XmlFields<'_> {
    let mut regular = Vec::new();
    let mut attrs = Vec::new();
    let mut text = None;
    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let is_attr = field.attrs.iter().any(|attr| attr.path().is_ident("attr"));
        let is_text = field.attrs.iter().any(|attr| attr.path().is_ident("text"));
        let rename = get_rename_attr(&field.attrs);
        let is_rename = rename.is_some();
        let renamed = rename.unwrap_or(field_name.to_string());
        match (is_attr, is_text) {
            (true, true) => panic!("#[attr] and #[text] are incompatible"),
            (true, false) => attrs.push(RenamedField {
                field_name,
                renamed,
            }),
            (false, true) if text.is_some() => panic!("only one #[text] field is allowed"),
            (false, true) if !regular.is_empty() => {
                panic!("#[text] is incompatible with regular fields")
            }
            (false, true) if is_rename => panic!("#[text] and #[rename] are incompatible"),
            (false, true) => text = Some(field_name),
            (false, false) if text.is_some() => {
                panic!("#[text] is incompatible with regular fields")
            }
            (false, false) => regular.push(RenamedField {
                field_name,
                renamed,
            }),
        }
    }
    XmlFields {
        regular,
        attrs,
        text,
    }
}

fn get_xml_variants(variants: &Punctuated<Variant, Comma>) -> Vec<RenamedField> {
    variants
        .iter()
        .map(|field| {
            let field_name = &field.ident;
            let renamed = get_rename_attr(&field.attrs).unwrap_or(field_name.to_string());
            RenamedField {
                field_name,
                renamed,
            }
        })
        .collect()
}
