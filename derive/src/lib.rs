#![allow(unused_imports)]

use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Attribute, Data, DataStruct, DeriveInput, Expr, Field, Fields, Ident, Lit, Variant};
use syn::{Type, parse_macro_input};

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
                quote! { ::nanoxml::derive::ser::SerXmlAsAttr::ser_as_text(&self.#text_field, xml)?; },
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

    let top_level_impl = quote! {
        impl ::nanoxml::derive::ser::SerXmlTopLevel for #name {
            const TAG_NAME: &'static str = #rename;
        }
    };

    let full_impl = quote! {
        #serxml_impl
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

    let as_attr_impl = quote! { impl ::nanoxml::derive::ser::SerXmlAsAttr for #name {} };

    let top_level_impl = quote! {
        impl ::nanoxml::derive::ser::SerXmlTopLevel for #name {
            const TAG_NAME: &'static str = #rename;
        }
    };

    let full_impl = quote! {
        #serxml_impl
        #as_attr_impl
        #top_level_impl
    };

    full_impl.into()
}

#[cfg(feature = "de")]
#[proc_macro_derive(DeXml, attributes(attr, rename, text))]
pub fn derive_dexml(input: TokenStream) -> TokenStream {
    use syn::DataEnum;

    let input = parse_macro_input!(input as DeriveInput);
    let rename = get_rename_attr(&input.attrs).unwrap_or(input.ident.to_string());
    match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => derive_dexml_struct(&input.ident, &rename, &fields.named),
        Data::Enum(DataEnum { variants, .. }) => {
            derive_dexml_enum(&input.ident, &rename, &variants)
        }
        _ => panic!("DeXml can only be derived for structs with named fields or enums"),
    }
}

#[cfg(feature = "de")]
fn derive_dexml_struct(
    name: &Ident,
    rename: &str,
    fields: &Punctuated<Field, Comma>,
) -> TokenStream {
    let xml_fields = get_xml_fields(fields);

    let field_init: Vec<_> = xml_fields
        .all
        .iter()
        .map(|field| {
            let TypedField { field_name, ty } = field;
            if is_vec(ty) {
                quote! { let mut #field_name = Vec::new(); }
            } else {
                quote! { let mut #field_name = None; }
            }
        })
        .collect();

    let attr_parse: Vec<_> = xml_fields
        .attrs
        .iter()
        .map(|field| {
            let RenamedField {
                field_name,
                renamed,
            } = field;
            quote! {
                #renamed => {
                    if #field_name.is_some() {
                        return Err(::nanoxml::de::XmlError::DuplicateField);
                    }
                    #field_name = Some(::nanoxml::derive::de::DeXmlAttr::de_xml_attr(__attr_value)?);
                }
            }
        })
        .collect();

    let regular_parse = match xml_fields.text {
        Some(text_field) => quote! {
            #text_field = Some(::nanoxml::derive::de::DeXmlAttr::de_xml_attr(parser.text()?)?);
            parser.tag_close("")?;
        },
        None => {
            let regular_parse: Vec<_> = xml_fields
                .regular
                .iter()
                .map(|field| {
                    let RenamedField {
                        field_name,
                        renamed,
                    } = field;
                    let ty = xml_fields.all.iter().find(|f| f.field_name.to_string() == field_name.to_string()).unwrap().ty;
                    if is_vec(ty) {
                        quote! {
                            #renamed => #field_name.push(::nanoxml::derive::de::DeXml::de_xml(parser)?),
                        }
                    } else {
                        quote! {
                            #renamed => {
                                if #field_name.is_some() {
                                    return Err(::nanoxml::de::XmlError::DuplicateField);
                                }
                                #field_name = Some(::nanoxml::derive::de::DeXml::de_xml(parser)?);
                            }
                        }
                    }
                })
                .collect();
            quote! {
                while let Ok((__tag)) = parser.tag_open_or_close(#rename)? {
                    match __tag {
                        #(#regular_parse)*
                        _ => return Err(::nanoxml::de::XmlError::InvalidField),
                    }
                }
            }
        }
    };

    let field_unwraps: Vec<_> = xml_fields
        .all
        .iter()
        .map(|field| {
            let TypedField { field_name, ty } = field;
            if !is_vec(ty) && !is_option(ty) {
                quote! { let #field_name = #field_name.ok_or(::nanoxml::de::XmlError::MissingField)?; }
            } else {
                quote! {}
            }
        })
        .collect();

    let field_returns: Vec<_> = xml_fields
        .all
        .iter()
        .map(|field| {
            let TypedField { field_name, .. } = field;
            quote! { #field_name, }
        })
        .collect();

    let dexml_impl = quote! {
        impl<'a> ::nanoxml::derive::de::DeXml<'a> for #name {
            fn de_xml(parser: &mut ::nanoxml::de::XmlParser<'a>) -> Result<Self, ::nanoxml::de::XmlError> {
                #(#field_init)*
                while let Ok((__attr_key, __attr_value)) = parser.attr_or_tag_open_end()? {
                    match __attr_key {
                        #(#attr_parse)*
                        _ => return Err(::nanoxml::de::XmlError::InvalidField),
                    }
                }
                #regular_parse
                #(#field_unwraps)*
                Ok(Self { #(#field_returns)* })
            }
        }
    };

    let top_level_impl = quote! {
        impl ::nanoxml::derive::de::DeXmlTopLevel<'_> for #name {
            const TAG_NAME: &'static str = #rename;
        }
    };

    let full_impl = quote! {
        #dexml_impl
        #top_level_impl
    };

    full_impl.into()
}

#[cfg(feature = "de")]
fn derive_dexml_enum(
    name: &Ident,
    rename: &str,
    variants: &Punctuated<Variant, Comma>,
) -> TokenStream {
    let variants = get_xml_variants(variants);

    if variants.is_empty() {
        panic!("empty enum cannot be deserialized");
    }

    let cases: Vec<_> = variants
        .iter()
        .map(|variant| {
            let RenamedField {
                field_name: variant_name,
                renamed,
            } = variant;
            quote! { if s == #renamed { Ok(Self::#variant_name) } }
        })
        .collect();

    let serxml_impl = quote! {
        impl ::nanoxml::derive::de::DeXmlAttr<'_> for #name {
            fn de_xml_attr(s: ::nanoxml::de::XmlStr<'_>) -> Result<Self, ::nanoxml::de::XmlError> {
                #(#cases else)*
                {
                    Err(::nanoxml::de::XmlError::InvalidVariant)
                }
            }
        }
    };

    let top_level_impl = quote! {
        impl ::nanoxml::derive::de::DeXmlTopLevel<'_> for #name {
            const TAG_NAME: &'static str = #rename;
        }
    };

    let full_impl = quote! {
        #serxml_impl
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
    all: Vec<TypedField<'a>>,
}

struct RenamedField<'a> {
    field_name: &'a Ident,
    renamed: String,
}

struct TypedField<'a> {
    field_name: &'a Ident,
    ty: &'a Type,
}

fn get_xml_fields(fields: &Punctuated<Field, Comma>) -> XmlFields<'_> {
    let mut regular = Vec::new();
    let mut attrs = Vec::new();
    let mut text = None;
    let mut all = Vec::new();
    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        all.push(TypedField {
            field_name,
            ty: &field.ty,
        });
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
        all,
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

fn is_option(ty: &Type) -> bool {
    check_type(
        ty,
        &["Option|", "std|option|Option|", "core|option|Option|"],
    )
}

fn is_vec(ty: &Type) -> bool {
    check_type(ty, &["Vec|", "std|vec|Vec|", "alloc|vec|Vec|"])
}

fn check_type(ty: &Type, valid: &[&str]) -> bool {
    use syn::{GenericArgument, Path, PathArguments, PathSegment};

    let path = match *ty {
        Type::Path(ref path) if path.qself.is_none() => &path.path,
        _ => return false,
    };

    let idents_of_path = path
        .segments
        .iter()
        .into_iter()
        .fold(String::new(), |mut acc, v| {
            acc.push_str(&v.ident.to_string());
            acc.push('|');
            acc
        });

    valid.iter().any(|s| &idents_of_path == *s)
}
