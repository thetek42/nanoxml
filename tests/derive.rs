use nanoxml::derive::{SerXml, SerXmlTopLevel};

#[derive(Debug, SerXml)]
struct User {
    id: Id,
    #[attr]
    name: String,
    password: String,
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
        password: String::from("123456"),
    };

    assert_eq!(
        user.serialize_to_string(),
        "<User name=\"admin\"><id>42</id><password>123456</password></User>"
    );
}
