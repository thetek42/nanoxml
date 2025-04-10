use std::net::Ipv4Addr;

use nanoxml::derive::{SerXml, SerXmlTopLevel};

#[derive(Debug, SerXml)]
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
    multi: Vec<i32>,
    ip: Ipv4Addr,
    role: Role,
}

#[derive(Debug, SerXml)]
#[allow(unused)]
enum Role {
    #[rename = "user"]
    User,
    #[rename = "mod"]
    Moderator,
    #[rename = "admin"]
    Admin,
}

#[derive(Debug, SerXml)]
struct Id {
    #[text]
    id: u64,
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

    assert_eq!(
        user.serialize_to_string(),
        "<user name=\"admin\" dname=\"Admin\" qux=\"456\"><id>42</id><pass>123456</pass><bar>123</bar><multi>-1</multi><multi>0</multi><multi>1</multi><ip>192.168.0.1</ip><role>admin</role></user>"
    );
}
