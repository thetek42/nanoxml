use core::net::{Ipv4Addr, Ipv6Addr};
use core::str::FromStr;

use crate::de::{XmlError, XmlParser, XmlStr};

pub use nanoxml_derive::DeXml;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::{borrow::Cow, string::String};

pub trait DeXml<'a>: Sized + 'a {
    fn de_xml(parser: &mut XmlParser<'a>) -> Result<Self, XmlError>;
}

pub trait DeXmlAttr<'a>: Sized + 'a {
    fn de_xml_attr(s: XmlStr<'a>) -> Result<Self, XmlError>;
}

impl<'a, T: DeXmlAttr<'a>> DeXml<'a> for T {
    fn de_xml(parser: &mut XmlParser<'a>) -> Result<Self, XmlError> {
        parser.tag_open_end()?;
        let s = parser.text()?;
        parser.tag_close("")?;
        Self::de_xml_attr(s)
    }
}

pub trait DeXmlTopLevel<'a>: DeXml<'a> {
    const TAG_NAME: &'static str;

    fn deserialize_str(s: &'a str) -> Result<Self, XmlError> {
        let mut parser = XmlParser::new(s)?;
        parser.tag_open_start(Self::TAG_NAME)?;
        let ret = Self::de_xml(&mut parser)?;
        parser.check_end()?;
        Ok(ret)
    }
}

macro_rules! impl_de_from_str {
    ($ty:ty) => {
        impl DeXmlAttr<'_> for $ty {
            fn de_xml_attr(s: XmlStr<'_>) -> Result<Self, XmlError> {
                FromStr::from_str(s.raw()).map_err(|_| XmlError::InvalidValue)
            }
        }
    };
}

impl_de_from_str!(u8);
impl_de_from_str!(u16);
impl_de_from_str!(u32);
impl_de_from_str!(u64);
impl_de_from_str!(i8);
impl_de_from_str!(i16);
impl_de_from_str!(i32);
impl_de_from_str!(i64);
impl_de_from_str!(f32);
impl_de_from_str!(f64);
impl_de_from_str!(bool);
impl_de_from_str!(Ipv4Addr);
impl_de_from_str!(Ipv6Addr);

impl<'a> DeXmlAttr<'a> for XmlStr<'a> {
    fn de_xml_attr(s: XmlStr<'a>) -> Result<Self, XmlError> {
        Ok(s)
    }
}

#[cfg(feature = "alloc")]
impl<'a> DeXmlAttr<'a> for Cow<'a, str> {
    fn de_xml_attr(s: XmlStr<'a>) -> Result<Self, XmlError> {
        Ok(s.parsed())
    }
}

#[cfg(feature = "alloc")]
impl DeXmlAttr<'_> for String {
    fn de_xml_attr(s: XmlStr<'_>) -> Result<Self, XmlError> {
        Ok(s.owned())
    }
}

#[cfg(feature = "heapless")]
impl<const N: usize> DeXmlAttr<'_> for heapless::String<N> {
    fn de_xml_attr(s: XmlStr<'_>) -> Result<Self, XmlError> {
        s.heapless().map_err(|_| XmlError::InvalidValue)
    }
}
