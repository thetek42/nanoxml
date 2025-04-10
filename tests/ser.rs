use nanoxml::ser::XmlBuilder;

#[test]
fn ser() {
    let mut s = String::new();
    let mut xml = XmlBuilder::new(&mut s);
    xml.tag_open("note").unwrap();
    xml.tag_with_text("to", "Tove").unwrap();
    xml.tag_with_text("from", "Ja&ni").unwrap();
    xml.tag_empty("empty").unwrap();
    xml.tag_open_attrs("body", &[("style", "italic"), ("font", "bäb\"öbü")])
        .unwrap();
    xml.text("Ħ€lłöWø®lð").unwrap();
    xml.tag_close("body").unwrap();
    xml.tag_close("note").unwrap();
    assert_eq!(
        s,
        "<note><to>Tove</to><from>Ja&amp;ni</from><empty/><body style=\"italic\" font=\"bäb&quot;öbü\">Ħ€lłöWø®lð</body></note>"
    );
}
