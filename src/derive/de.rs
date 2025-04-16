use core::mem::MaybeUninit;
use core::net::{Ipv4Addr, Ipv6Addr};
use core::str::FromStr;

use crate::de::{XmlError, XmlParser, XmlStr};

pub use nanoxml_derive::DeXml;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::{borrow::Cow, string::String, vec::Vec};

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

pub trait DeXmlSeq<'a>: Sized + 'a {
    type Intermediate;

    fn new_seq() -> Self::Intermediate;
    fn push_item(this: &mut Self::Intermediate, parser: &mut XmlParser<'a>)
    -> Result<(), XmlError>;
    fn finish(this: Self::Intermediate) -> Result<Self, XmlError>;
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

impl<'a, T: DeXml<'a>> DeXmlSeq<'a> for Vec<T> {
    type Intermediate = Self;

    fn new_seq() -> Self::Intermediate {
        Self::new()
    }

    fn push_item(
        this: &mut Self::Intermediate,
        parser: &mut XmlParser<'a>,
    ) -> Result<(), XmlError> {
        Ok(this.push(T::de_xml(parser)?))
    }

    fn finish(this: Self::Intermediate) -> Result<Self, XmlError> {
        Ok(this)
    }
}

#[cfg(feature = "heapless")]
impl<'a, T: DeXml<'a>, const N: usize> DeXmlSeq<'a> for heapless::Vec<T, N> {
    type Intermediate = Self;

    fn new_seq() -> Self::Intermediate {
        Self::new()
    }

    fn push_item(
        this: &mut Self::Intermediate,
        parser: &mut XmlParser<'a>,
    ) -> Result<(), XmlError> {
        this.push(T::de_xml(parser)?)
            .map_err(|_| XmlError::SeqOverflow)
    }

    fn finish(this: Self::Intermediate) -> Result<Self, XmlError> {
        Ok(this)
    }
}

// this stuff is required because rust stoopid
trait UninitArray<T, const N: usize> {
    const UNINIT_ELEM: MaybeUninit<T> = MaybeUninit::uninit();
    const UNINIT_ARRAY: [MaybeUninit<T>; N] = [Self::UNINIT_ELEM; N];
}

impl<T, const N: usize> UninitArray<T, N> for [T; N] {}

impl<'a, T: DeXml<'a>, const N: usize> DeXmlSeq<'a> for [T; N] {
    type Intermediate = ([MaybeUninit<T>; N], usize);

    fn new_seq() -> Self::Intermediate {
        (Self::UNINIT_ARRAY, 0)
    }

    fn push_item(
        this: &mut Self::Intermediate,
        parser: &mut XmlParser<'a>,
    ) -> Result<(), XmlError> {
        if this.1 >= N {
            Err(XmlError::SeqOverflow)
        } else {
            this.0[this.1].write(T::de_xml(parser)?);
            this.1 += 1;
            Ok(())
        }
    }

    fn finish(this: Self::Intermediate) -> Result<Self, XmlError> {
        if this.1 == N {
            Ok(unsafe { MaybeUninit::array_assume_init(this.0) })
        } else {
            Err(XmlError::SeqUnderflow)
        }
    }
}
