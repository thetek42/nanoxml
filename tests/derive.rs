#![allow(unused)]

use std::borrow::Cow;
use std::net::Ipv4Addr;

use nanoxml::de::XmlStr;
use nanoxml::derive::de::{DeXml, DeXmlTopLevel};
use nanoxml::derive::ser::{SerXml, SerXmlTopLevel};

#[derive(Debug, DeXml, PartialEq, SerXml)]
#[rename = "user"]
struct User {
    id: Id,
    #[attr]
    name: String,
    #[attr]
    #[rename = "dname"]
    display_name: String,
    #[rename = "pass"]
    password: String,
    foo: Option<String>,
    bar: Option<String>,
    #[attr]
    baz: Option<String>,
    #[attr]
    qux: Option<String>,
    #[seq]
    multi: Vec<i32>,
    ip: Ipv4Addr,
    role: Role,
}

#[derive(Debug, DeXml, PartialEq, SerXml)]
enum Role {
    #[rename = "user"]
    User,
    #[rename = "mod"]
    Moderator,
    #[rename = "admin"]
    Admin,
}

#[derive(Debug, DeXml, PartialEq, SerXml)]
struct Id {
    #[text]
    id: u64,
}

#[derive(Debug, DeXml, PartialEq, SerXml)]
struct Lifetimed<'a> {
    str: XmlStr<'a>,
    cow: Cow<'a, str>,
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
    };

    let xml = user.serialize_to_string();
    assert_eq!(
        xml,
        "<user name=\"admin\" dname=\"Admin\" qux=\"456\"><id>42</id><pass>123456</pass><bar>123</bar><multi>-1</multi><multi>0</multi><multi>1</multi><ip>192.168.0.1</ip><role>admin</role></user>"
    );

    let reconstructed = User::deserialize_str(&xml).unwrap();
    assert_eq!(user, reconstructed);

    let lifetimed_xml = "<Lifetimed><str>foo</str><cow>bar</cow></Lifetimed>";
    let lifetimed = Lifetimed::deserialize_str(lifetimed_xml).unwrap();
    assert_eq!(lifetimed.str, "foo");
    assert_eq!(lifetimed.cow, "bar");
    let xml_reconstructed = lifetimed.serialize_to_string();
    assert_eq!(lifetimed_xml, xml_reconstructed);
}
