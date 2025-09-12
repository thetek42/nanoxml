#![allow(unused)]

use std::borrow::Cow;
use std::net::Ipv4Addr;

use nanoxml::de::XmlStr;
use nanoxml::derive::de::{DeXml, DeXmlTopLevel};
use nanoxml::derive::ser::{RawXml, SerXml, SerXmlTopLevel};

#[derive(Debug, DeXml, PartialEq, SerXml)]
#[nanoxml(rename = "user")]
struct User {
    id: Id,
    #[nanoxml(attr)]
    name: String,
    #[nanoxml(attr, rename = "dname")]
    display_name: String,
    #[nanoxml(rename = "pass")]
    password: String,
    foo: Option<String>,
    bar: Option<String>,
    #[nanoxml(attr)]
    baz: Option<String>,
    #[nanoxml(attr)]
    qux: Option<String>,
    #[nanoxml(seq)]
    multi: Vec<i32>,
    ip: Ipv4Addr,
    role: Role,
    #[nanoxml(skip_ser, default_de = "fourtytwo")]
    skip: i32,
    empty: String,
}

#[derive(Debug, DeXml, PartialEq, SerXml)]
enum Role {
    #[nanoxml(rename = "user")]
    User,
    #[nanoxml(rename = "mod")]
    Moderator,
    #[nanoxml(rename = "admin")]
    Admin,
}

#[derive(Debug, DeXml, PartialEq, SerXml)]
struct Id {
    #[nanoxml(text)]
    id: u64,
}

#[derive(Debug, DeXml, PartialEq, SerXml)]
struct Lifetimed<'a> {
    str: XmlStr<'a>,
    cow: Cow<'a, str>,
}

#[derive(Debug, PartialEq, SerXml)]
struct SerOnly {
    text: String,
    raw: RawXml,
}

#[test]
fn derive() {
    let user = User {
        id: Id { id: 42 },
        name: String::from("admin"),
        display_name: String::from("Admin"),
        password: String::from("123456"),
        foo: None,
        bar: Some(String::from("123")),
        baz: None,
        qux: Some(String::from("456")),
        multi: vec![-1, 0, 1],
        ip: Ipv4Addr::new(192, 168, 0, 1),
        role: Role::Admin,
        skip: 42,
        empty: String::new(),
    };

    let xml = user.serialize_to_string();
    assert_eq!(
        xml,
        "<user name=\"admin\" dname=\"Admin\" qux=\"456\"><id>42</id><pass>123456</pass><bar>123</bar><multi>-1</multi><multi>0</multi><multi>1</multi><ip>192.168.0.1</ip><role>admin</role><empty></empty></user>"
    );

    let reconstructed = User::deserialize_str(&xml).unwrap();
    assert_eq!(user, reconstructed);

    let lifetimed_xml = "<Lifetimed><str>foo</str><cow>bar</cow></Lifetimed>";
    let lifetimed = Lifetimed::deserialize_str(lifetimed_xml).unwrap();
    assert_eq!(lifetimed.str, "foo");
    assert_eq!(lifetimed.cow, "bar");
    let xml_reconstructed = lifetimed.serialize_to_string();
    assert_eq!(lifetimed_xml, xml_reconstructed);

    let ser_only = SerOnly {
        text: String::from("<Foo>Bar&Baz</Foo>"),
        raw: String::from("<Foo>Bar&Baz</Foo>").into(),
    };
    let ser_only_xml = ser_only.serialize_to_string();
    assert_eq!(
        ser_only_xml,
        "<SerOnly><text>&lt;Foo&gt;Bar&amp;Baz&lt;/Foo&gt;</text><raw><Foo>Bar&Baz</Foo></raw></SerOnly>",
    );
}

fn fourtytwo() -> i32 {
    42
}
