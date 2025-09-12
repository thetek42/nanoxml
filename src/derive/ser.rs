use core::fmt::Error as FmtError;
use core::fmt::Result as FmtResult;
use core::fmt::Write;
use core::net::IpAddr;
use core::net::Ipv4Addr;
use core::net::Ipv6Addr;

use crate::de::XmlStr;
use crate::ser::XmlBuilder;

pub use nanoxml_derive::SerXml;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::{borrow::Cow, string::String, vec::Vec};

pub trait SerXml {
    fn ser_body<W: Write>(&self, xml: &mut XmlBuilder<'_, W>) -> FmtResult;
    fn ser_attrs<W: Write>(&self, xml: &mut XmlBuilder<'_, W>) -> FmtResult;

    fn ser_xml<W: Write>(&self, xml: &mut XmlBuilder<'_, W>, tag_name: &str) -> FmtResult {
        xml.tag_open_start(tag_name)?;
        self.ser_attrs(xml)?;
        xml.tag_open_end()?;
        self.ser_body(xml)?;
        xml.tag_close(tag_name)
    }
}

pub trait SerXmlAsAttr: SerXml {
    fn ser_as_attr<W: Write>(&self, xml: &mut XmlBuilder<'_, W>, attr_key: &str) -> FmtResult {
        xml.attr_start(attr_key)?;
        self.ser_body(xml)?;
        xml.attr_end()
    }

    fn ser_as_text<W: Write>(&self, xml: &mut XmlBuilder<'_, W>) -> FmtResult {
        self.ser_body(xml)
    }
}

pub trait SerXmlTopLevel: SerXml {
    const TAG_NAME: &'static str;

    fn serialize<W: Write>(&self, writer: &mut W) -> FmtResult {
        let mut xml = XmlBuilder::new(writer);
        self.ser_xml(&mut xml, Self::TAG_NAME)
    }

    #[cfg(feature = "alloc")]
    fn serialize_to_string(&self) -> String {
        let mut s = String::new();
        self.serialize(&mut s).unwrap();
        s
    }
}

macro_rules! impl_ser_primitive {
    ($ty:ty) => {
        impl SerXml for $ty {
            fn ser_body<W: Write>(&self, xml: &mut XmlBuilder<'_, W>) -> FmtResult {
                write!(xml.writer, "{}", self)
            }
            fn ser_attrs<W: Write>(&self, xml: &mut XmlBuilder<'_, W>) -> FmtResult {
                _ = xml;
                Ok(())
            }
        }
        impl SerXmlAsAttr for $ty {}
    };
}

impl_ser_primitive!(i8);
impl_ser_primitive!(i16);
impl_ser_primitive!(i32);
impl_ser_primitive!(i64);
impl_ser_primitive!(isize);
impl_ser_primitive!(u8);
impl_ser_primitive!(u16);
impl_ser_primitive!(u32);
impl_ser_primitive!(u64);
impl_ser_primitive!(usize);
impl_ser_primitive!(f32);
impl_ser_primitive!(f64);
impl_ser_primitive!(bool);
impl_ser_primitive!(IpAddr);
impl_ser_primitive!(Ipv4Addr);
impl_ser_primitive!(Ipv6Addr);

macro_rules! impl_ser_str {
    ($ty:ty) => {
        impl SerXml for $ty {
            fn ser_body<W: Write>(&self, xml: &mut XmlBuilder<'_, W>) -> FmtResult {
                xml.text(self.as_ref())
            }
            fn ser_attrs<W: Write>(&self, xml: &mut XmlBuilder<'_, W>) -> FmtResult {
                _ = xml;
                Ok(())
            }
        }
        impl SerXmlAsAttr for $ty {}
    };
}

impl_ser_str!(str);

#[cfg(feature = "alloc")]
impl_ser_str!(String);

#[cfg(feature = "alloc")]
impl<'a> SerXml for Cow<'a, str> {
    fn ser_body<W: Write>(&self, xml: &mut XmlBuilder<'_, W>) -> FmtResult {
        xml.text(self.as_ref())
    }

    fn ser_attrs<W: Write>(&self, xml: &mut XmlBuilder<'_, W>) -> FmtResult {
        _ = xml;
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<'a> SerXmlAsAttr for Cow<'a, str> {}

#[cfg(feature = "de")]
impl<'a> SerXml for XmlStr<'a> {
    fn ser_body<W: Write>(&self, xml: &mut XmlBuilder<'_, W>) -> FmtResult {
        write!(xml.writer, "{}", self.raw())
    }

    fn ser_attrs<W: Write>(&self, xml: &mut XmlBuilder<'_, W>) -> FmtResult {
        _ = xml;
        Ok(())
    }
}

#[cfg(feature = "de")]
impl<'a> SerXmlAsAttr for XmlStr<'a> {}

impl<T: SerXml> SerXml for Option<T> {
    fn ser_body<W: Write>(&self, xml: &mut XmlBuilder<'_, W>) -> FmtResult {
        match self {
            Some(t) => t.ser_body(xml),
            None => Ok(()),
        }
    }

    fn ser_attrs<W: Write>(&self, xml: &mut XmlBuilder<'_, W>) -> FmtResult {
        match self {
            Some(t) => t.ser_attrs(xml),
            None => Ok(()),
        }
    }

    fn ser_xml<W: Write>(&self, xml: &mut XmlBuilder<'_, W>, tag_name: &str) -> FmtResult {
        match self {
            Some(t) => t.ser_xml(xml, tag_name),
            None => Ok(()),
        }
    }
}

impl<T: SerXmlAsAttr> SerXmlAsAttr for Option<T> {
    fn ser_as_attr<W: Write>(&self, xml: &mut XmlBuilder<'_, W>, attr_key: &str) -> FmtResult {
        match self {
            Some(t) => {
                xml.attr_start(attr_key)?;
                t.ser_body(xml)?;
                xml.attr_end()
            }
            None => Ok(()),
        }
    }
}

macro_rules! impl_ser_iter {
    ($ty:ty) => {
        impl<T: SerXml> SerXml for $ty {
            fn ser_body<W: Write>(&self, xml: &mut XmlBuilder<'_, W>) -> FmtResult {
                _ = xml;
                Err(FmtError)
            }
            fn ser_attrs<W: Write>(&self, xml: &mut XmlBuilder<'_, W>) -> FmtResult {
                _ = xml;
                Err(FmtError)
            }
            fn ser_xml<W: Write>(&self, xml: &mut XmlBuilder<'_, W>, tag_name: &str) -> FmtResult {
                for item in self.iter() {
                    item.ser_xml(xml, tag_name)?;
                }
                Ok(())
            }
        }
    };
}

macro_rules! impl_ser_iter_n {
    ($ty:ty) => {
        impl<T: SerXml, const N: usize> SerXml for $ty {
            fn ser_body<W: Write>(&self, xml: &mut XmlBuilder<'_, W>) -> FmtResult {
                _ = xml;
                Err(FmtError)
            }
            fn ser_attrs<W: Write>(&self, xml: &mut XmlBuilder<'_, W>) -> FmtResult {
                _ = xml;
                Err(FmtError)
            }
            fn ser_xml<W: Write>(&self, xml: &mut XmlBuilder<'_, W>, tag_name: &str) -> FmtResult {
                for item in self.iter() {
                    item.ser_xml(xml, tag_name)?;
                }
                Ok(())
            }
        }
    };
}

impl_ser_iter!([T]);
impl_ser_iter_n!([T; N]);

#[cfg(feature = "alloc")]
impl_ser_iter!(Vec<T>);

#[cfg(feature = "heapless")]
impl_ser_iter_n!(heapless::Vec<T, N>);

#[cfg(feature = "heapless")]
impl<const N: usize> SerXml for heapless::String<N> {
    fn ser_body<W: Write>(&self, xml: &mut XmlBuilder<'_, W>) -> FmtResult {
        write!(xml.writer, "{}", self)
    }

    fn ser_attrs<W: Write>(&self, xml: &mut XmlBuilder<'_, W>) -> FmtResult {
        _ = xml;
        Ok(())
    }
}

#[cfg(feature = "heapless")]
impl<const N: usize> SerXmlAsAttr for heapless::String<N> {}

impl<T: SerXml + ?Sized> SerXml for &T {
    fn ser_attrs<W: Write>(&self, xml: &mut XmlBuilder<'_, W>) -> FmtResult {
        T::ser_attrs(*self, xml)
    }

    fn ser_body<W: Write>(&self, xml: &mut XmlBuilder<'_, W>) -> FmtResult {
        T::ser_body(*self, xml)
    }
}

impl<T: SerXmlAsAttr + ?Sized> SerXmlAsAttr for &T {}

#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RawXml(pub String);

#[cfg(feature = "alloc")]
impl From<String> for RawXml {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[cfg(feature = "alloc")]
impl SerXml for RawXml {
    fn ser_body<W: Write>(&self, xml: &mut XmlBuilder<'_, W>) -> FmtResult {
        xml.write_str(&self.0)
    }
    fn ser_attrs<W: Write>(&self, xml: &mut XmlBuilder<'_, W>) -> FmtResult {
        _ = xml;
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl SerXmlAsAttr for RawXml {}
