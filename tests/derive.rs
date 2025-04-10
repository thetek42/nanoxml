use nanoxml::derive::{SerXml, SerXmlTopLevel};

#[derive(Debug, SerXml)]
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
    };

    assert_eq!(
        user.serialize_to_string(),
        "<User name=\"admin\" dname=\"Admin\" qux=\"456\"><id>42</id><pass>123456</pass><bar>123</bar></User>"
    );
}
