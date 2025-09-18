#![allow(unused)]

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Attribute, Data, DataEnum, DataStruct, DeriveInput, Expr, Field};
use syn::{Fields, GenericParam, Generics, Ident, Lifetime, LifetimeParam};
use syn::{Lit, LitStr, Type, Variant};

#[cfg(feature = "ser")]
#[proc_macro_derive(SerXml, attributes(nanoxml))]
pub fn derive_serxml(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let rename = get_rename_attr(&input.attrs).unwrap_or(input.ident.to_string());
    match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => derive_serxml_struct(&input.ident, &rename, &fields.named, input.generics),
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
    generics: Generics,
) -> TokenStream {
    let xml_fields = get_xml_fields(fields);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let ser_text = xml_fields
        .iter()
        .find(|f| f.field_kind == FieldKind::Text)
        .filter(|f| !f.skip_ser)
        .map(|f| {
            let field_name = f.field_name;
            quote! { ::nanoxml::derive::ser::SerXmlAsAttr::ser_as_text(&self.#field_name, __xml)?; }
        });

    let ser_regular = xml_fields
        .iter()
        .filter(|f| f.field_kind == FieldKind::Regular)
        .filter(|f| !f.skip_ser)
        .map(|f| {
            let field_name = f.field_name;
            let renamed = &f.renamed;
            quote! { ::nanoxml::derive::ser::SerXml::ser_xml(&self.#field_name, __xml, #renamed)?; }
        });

    let ser_attrs = xml_fields
        .iter()
        .filter(|f| f.field_kind == FieldKind::Attr)
        .filter(|f| !f.skip_ser)
        .map(|f| {
            let field_name = f.field_name;
            let renamed = &f.renamed;
            quote! { ::nanoxml::derive::ser::SerXmlAsAttr::ser_as_attr(&self.#field_name, __xml, #renamed)?; }
        });

    let serxml_impl = quote! {
        impl #impl_generics ::nanoxml::derive::ser::SerXml for #name #ty_generics #where_clause {
            fn ser_body<W: ::core::fmt::Write>(&self, __xml: &mut ::nanoxml::ser::XmlBuilder<'_, W>) -> ::core::fmt::Result {
                #ser_text
                #(#ser_regular)*
                Ok(())
            }

            fn ser_attrs<W: ::core::fmt::Write>(&self, __xml: &mut ::nanoxml::ser::XmlBuilder<'_, W>) -> ::core::fmt::Result {
                #(#ser_attrs)*
                Ok(())
            }
        }
    };

    let attr_impl = xml_fields.iter().all(|f| f.field_kind == FieldKind::Text).then(|| quote !{
        impl #impl_generics ::nanoxml::derive::ser::SerXmlAsAttr for #name #ty_generics #where_clause {}
    });

    let top_level_impl = quote! {
        impl #impl_generics ::nanoxml::derive::ser::SerXmlTopLevel for #name #ty_generics #where_clause {
            const TAG_NAME: &'static str = #rename;
        }
    };

    let full_impl = quote! {
        #serxml_impl
        #attr_impl
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
        .map(|v| {
            let variant_name = v.variant_name;
            let renamed = &v.renamed;
            quote! { Self::#variant_name => __xml.text(#renamed), }
        })
        .collect();

    let serxml_impl = quote! {
        impl ::nanoxml::derive::ser::SerXml for #name {
            fn ser_body<W: ::core::fmt::Write>(&self, __xml: &mut ::nanoxml::ser::XmlBuilder<'_, W>) -> ::core::fmt::Result {
                match self {
                    #(#cases)*
                }
            }

            fn ser_attrs<W: ::core::fmt::Write>(&self, __xml: &mut ::nanoxml::ser::XmlBuilder<'_, W>) -> ::core::fmt::Result {
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
#[proc_macro_derive(DeXml, attributes(nanoxml))]
pub fn derive_dexml(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let rename = get_rename_attr(&input.attrs).unwrap_or(input.ident.to_string());
    match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => derive_dexml_struct(&input.ident, &rename, &fields.named, input.generics),
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
    generics: Generics,
) -> TokenStream {
    use quote::format_ident;

    let xml_fields = get_xml_fields(fields);

    let mut generics_clone = generics.clone();
    let lifetime_param = match generics.lifetimes().next() {
        Some(lt) => quote! { <#lt> },
        None => {
            generics_clone
                .params
                .push(GenericParam::Lifetime(LifetimeParam::new(Lifetime::new(
                    "'a",
                    Span::call_site(),
                ))));
            quote! { <'a> }
        }
    };
    let (impl_generics, _, _) = generics_clone.split_for_impl();
    let (_, ty_generics, where_clause) = generics.split_for_impl();

    let field_init = xml_fields.iter().map(|f| {
        let field_name = f.field_name;
        let real_type = f.real_type;
        match f.field_type {
            FieldType::Seq => quote! { let mut #field_name = <#real_type as ::nanoxml::derive::de::DeXmlSeq>::new_seq(); },
            _ => quote! { let mut #field_name = None; },
        }
    });

    let de_attr = xml_fields
        .iter()
        .filter(|f| f.field_kind == FieldKind::Attr)
        .map(|f| {
            let field_name = f.field_name;
            let renamed = &f.renamed;
            quote! {
                #renamed => {
                    if #field_name.is_some() {
                        return Err(::nanoxml::de::XmlError::DuplicateField);
                    }
                    #field_name = Some(::nanoxml::derive::de::DeXmlAttr::de_xml_attr(__attr_value)?);
                }
            }
        });

    let de_text = xml_fields
        .iter()
        .find(|f| f.field_kind == FieldKind::Text)
        .map(|f| {
            let field_name = f.field_name;
            quote! {
                #field_name = Some(::nanoxml::derive::de::DeXmlAttr::de_xml_attr(__parser.text()?)?);
                __parser.tag_close()?;
            }
        });

    let de_regular = xml_fields
        .iter()
        .filter(|f| f.field_kind == FieldKind::Regular)
        .map(|f| {
            let field_name = f.field_name;
            let real_type = f.real_type;
            let renamed = &f.renamed;
            match f.field_type {
                FieldType::Seq => quote! {
                    #renamed => <#real_type as ::nanoxml::derive::de::DeXmlSeq>::push_item(&mut #field_name, __parser)?,
                },
                _ => quote! {
                    #renamed => {
                        if #field_name.is_some() {
                            return Err(::nanoxml::de::XmlError::DuplicateField);
                        }
                        #field_name = Some(::nanoxml::derive::de::DeXml::de_xml(__parser)?);
                    }
                },
            }
        });

    let de_body = match de_text {
        Some(de_text) => de_text,
        None => quote! {
            while let Ok((__tag)) = __parser.tag_open_or_close()? {
                match __tag {
                    #(#de_regular)*
                    _ => return Err(::nanoxml::de::XmlError::InvalidField),
                }
            }
        },
    };

    let field_unwraps: Vec<_> = xml_fields
        .iter()
        .map(|f| {
            let field_name = f.field_name;
            match f.field_type {
                FieldType::Regular => match &f.default_de {
                    None => quote! { #field_name: #field_name.ok_or(::nanoxml::de::XmlError::MissingField)?, },
                    Some(None) => quote! { #field_name: #field_name.unwrap_or_default(), },
                    Some(Some(func)) => {
                        let func = format_ident!("{func}");
                        quote! { #field_name: #field_name.unwrap_or_else(#func), }
                    }
                }
                FieldType::Option => quote! { #field_name, },
                FieldType::Seq => quote! { #field_name: ::nanoxml::derive::de::DeXmlSeq::finish(#field_name)?, },
            }
        })
        .collect();

    let dexml_impl = quote! {
        impl #lifetime_param ::nanoxml::derive::de::DeXml #lifetime_param for #name #ty_generics #where_clause {
            fn de_xml(__parser: &mut ::nanoxml::de::XmlParser<'a>) -> Result<Self, ::nanoxml::de::XmlError> {
                #(#field_init)*
                while let Ok((__attr_key, __attr_value)) = __parser.attr_or_tag_open_end()? {
                    match __attr_key {
                        #(#de_attr)*
                        _ => return Err(::nanoxml::de::XmlError::InvalidField),
                    }
                }
                #de_body
                Ok(Self { #(#field_unwraps)* })
            }
        }
    };

    let top_level_impl = quote! {
        impl #impl_generics ::nanoxml::derive::de::DeXmlTopLevel #lifetime_param for #name #ty_generics #where_clause {
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
        .map(|v| {
            let variant_name = v.variant_name;
            let renamed = &v.renamed;
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
    let mut renamed = None;
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("nanoxml")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                if renamed.is_some() {
                    panic!("duplicate rename attr")
                }
                let value = meta.value().expect("rename requires value");
                let lit: LitStr = value.parse().expect("rename requires atr value");
                renamed = Some(lit.value());
            } else {
                panic!("invalid nanoxml attr");
            }
            Ok(())
        })
        .unwrap();
    }
    renamed
}

struct XmlField<'a> {
    field_name: &'a Ident,
    field_type: FieldType,
    field_kind: FieldKind,
    real_type: &'a Type,
    renamed: String,
    skip_ser: bool,
    default_de: Option<Option<String>>,
}

struct XmlVariant<'a> {
    variant_name: &'a Ident,
    renamed: String,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum FieldKind {
    Regular,
    Text,
    Attr,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum FieldType {
    Regular,
    Option,
    Seq,
}

fn get_xml_fields(fields: &Punctuated<Field, Comma>) -> Vec<XmlField<'_>> {
    let mut ret = Vec::<XmlField<'_>>::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let real_type = &field.ty;

        let mut renamed = None;
        let mut is_seq = false;
        let mut is_attr = false;
        let mut is_text = false;
        let mut skip_ser = false;
        let mut default_de = None;

        for attr in field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("nanoxml"))
        {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    if renamed.is_some() {
                        panic!("duplicate rename attr")
                    }
                    let value = meta.value().expect("rename requires value");
                    let lit: LitStr = value.parse().expect("rename requires atr value");
                    renamed = Some(lit.value());
                } else if meta.path.is_ident("seq") {
                    is_seq = true;
                } else if meta.path.is_ident("attr") {
                    is_attr = true;
                } else if meta.path.is_ident("text") {
                    is_text = true;
                } else if meta.path.is_ident("skip_ser") {
                    skip_ser = true;
                } else if meta.path.is_ident("default_de") {
                    if default_de.is_some() {
                        panic!("duplicate default_de attr")
                    }
                    match meta.value() {
                        Ok(value) => {
                            let lit: LitStr = value.parse().expect("default_de requires str value");
                            default_de = Some(Some(lit.value()));
                        }
                        Err(_) => default_de = Some(None),
                    }
                } else {
                    panic!("invalid nanoxml attr");
                }
                Ok(())
            })
            .unwrap();
        }

        let field_type = if is_seq {
            FieldType::Seq
        } else if is_option(real_type) {
            FieldType::Option
        } else {
            FieldType::Regular
        };

        if default_de.is_some() && field_type != FieldType::Regular {
            panic!("default_de only works for non-option, non-seq fields");
        }

        let field_kind = match (is_attr, is_text) {
            (true, true) => {
                panic!("#[attr] and #[text] on the same field are incompatible");
            }
            (false, true) if ret.iter().any(|f| f.field_kind == FieldKind::Text) => {
                panic!("only one #[text] field is allowed");
            }
            (false, true) if ret.iter().any(|f| f.field_kind == FieldKind::Regular) => {
                panic!("#[text] is incompatible with regular fields");
            }
            (false, true) if renamed.is_some() => {
                panic!("#[text] and #[rename] on the same field are incompatible");
            }
            (false, false) if ret.iter().any(|f| f.field_kind == FieldKind::Text) => {
                panic!("#[text] is incompatible with regular fields");
            }
            (true, false) => FieldKind::Attr,
            (false, true) => FieldKind::Text,
            (false, false) => FieldKind::Regular,
        };

        let renamed = renamed.unwrap_or_else(|| field_name.to_string());

        ret.push(XmlField {
            field_name,
            field_type,
            field_kind,
            real_type,
            renamed,
            skip_ser,
            default_de,
        });
    }

    ret
}

fn get_xml_variants(variants: &Punctuated<Variant, Comma>) -> Vec<XmlVariant<'_>> {
    let mut ret = Vec::<XmlVariant<'_>>::new();

    for variant in variants {
        let variant_name = &variant.ident;

        let mut renamed = variant_name.to_string();

        for attr in variant
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("nanoxml"))
        {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    let value = meta.value().expect("rename requires value");
                    let lit: LitStr = value.parse().expect("rename requires atr value");
                    renamed = lit.value();
                } else {
                    panic!("invalid nanoxml attr");
                }
                Ok(())
            })
            .unwrap();
        }

        ret.push(XmlVariant {
            variant_name,
            renamed,
        });
    }

    ret
}

fn is_option(ty: &Type) -> bool {
    check_type(
        ty,
        &["Option|", "std|option|Option|", "core|option|Option|"],
    )
}

fn check_type(ty: &Type, valid: &[&str]) -> bool {
    let path = match *ty {
        Type::Path(ref path) if path.qself.is_none() => &path.path,
        _ => return false,
    };

    let idents_of_path = path.segments.iter().fold(String::new(), |mut acc, v| {
        acc.push_str(&v.ident.to_string());
        acc.push('|');
        acc
    });

    valid.iter().any(|s| idents_of_path == *s)
}
