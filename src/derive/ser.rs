use core::fmt::Error as FmtError;
use core::fmt::Result as FmtResult;
use core::fmt::Write;
use core::net::Ipv4Addr;
use core::net::Ipv6Addr;

use crate::ser::XmlBuilder;

pub use nanoxml_derive::SerXml;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

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
impl_ser_primitive!(u8);
impl_ser_primitive!(u16);
impl_ser_primitive!(u32);
impl_ser_primitive!(u64);
impl_ser_primitive!(f32);
impl_ser_primitive!(f64);
impl_ser_primitive!(bool);
impl_ser_primitive!(str);
impl_ser_primitive!(Ipv4Addr);
impl_ser_primitive!(Ipv6Addr);

#[cfg(feature = "alloc")]
impl_ser_primitive!(String);

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
            Some(t) => {
                xml.tag_open_start(tag_name)?;
                t.ser_attrs(xml)?;
                xml.tag_open_end()?;
                t.ser_body(xml)?;
                xml.tag_close(tag_name)
            }
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
