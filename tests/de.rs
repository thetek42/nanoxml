use std::borrow::Cow;

use nanoxml::de::XmlParser;

#[test]
fn de() {
    let xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <note>
          <to>To&foo;ve</to>
          <from>Ja&amp;ni</from>
          <empty />
          <body style="italic" font="bäb&quot;ö&foo;bü">Ħ€lłöWø®lð</body>
        </note>
    "#;

    let mut xml = XmlParser::new(xml).unwrap();
    xml.tag_open_start("note").unwrap();
    xml.tag_open_end().unwrap();
    xml.tag_open_start("to").unwrap();
    xml.tag_open_end().unwrap();
    let to = xml.text().unwrap();
    xml.tag_close().unwrap();
    xml.tag_open_start("from").unwrap();
    xml.tag_open_end().unwrap();
    let from = xml.text().unwrap();
    xml.tag_close().unwrap();
    xml.tag_open_start("empty").unwrap();
    xml.tag_open_end().unwrap();
    xml.tag_close().unwrap();
    xml.tag_open_start("body").unwrap();
    let (style_key, style_value) = xml.attr().unwrap();
    let (font_key, font_value) = xml.attr().unwrap();
    xml.tag_open_end().unwrap();
    let body = xml.text().unwrap();
    xml.tag_close().unwrap();
    xml.tag_close().unwrap();
    xml.check_end().unwrap();

    assert_eq!(to.raw(), "To&foo;ve");
    assert!(matches!(to.parsed(), Cow::Borrowed("To&foo;ve")));
    assert_eq!(from.raw(), "Ja&amp;ni");
    assert!(matches!(from.parsed(), Cow::Owned(s) if s == "Ja&ni"));
    assert_eq!(style_key, "style");
    assert_eq!(style_value.raw(), "italic");
    assert!(matches!(style_value.parsed(), Cow::Borrowed("italic")));
    assert_eq!(font_key, "font");
    assert_eq!(font_value.raw(), "bäb&quot;ö&foo;bü");
    assert!(matches!(font_value.parsed(), Cow::Owned(s) if s == "bäb\"ö&foo;bü"));
    assert_eq!(body.raw(), "Ħ€lłöWø®lð");
    assert!(matches!(body.parsed(), Cow::Borrowed("Ħ€lłöWø®lð")));
}
