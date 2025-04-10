// TODO: derives

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::{borrow::Cow, string::String};

pub struct XmlParser<'a> {
    pub(crate) s: &'a str,
    pub(crate) n: usize,
    pub(crate) in_tag: bool,
    pub(crate) selfclose: bool,
}

impl<'a> XmlParser<'a> {
    pub fn new(s: &'a str) -> Result<Self, XmlError> {
        let s = skip_xml_header(s.trim())?;
        Ok(Self {
            s,
            n: 0,
            in_tag: false,
            selfclose: false,
        })
    }

    pub fn next_token(&mut self) -> Result<Option<XmlToken<'a>>, XmlError> {
        if self.selfclose {
            self.selfclose = false;
            return Ok(Some(XmlToken::TagClose("")));
        }

        self.consume_whitespace();
        match self.consume_ascii() {
            Some(b'<') => {
                let close = self.opt_consume_ascii(b'/').is_some();
                let identifier = self.consume_identifier()?;
                Ok(Some(match close {
                    true => {
                        self.expect_ascii(b'>')?;
                        XmlToken::TagClose(identifier)
                    }
                    false => {
                        self.in_tag = true;
                        XmlToken::TagOpenStart(identifier)
                    }
                }))
            }
            Some(b'>') => {
                self.in_tag = false;
                Ok(Some(XmlToken::TagOpenEnd))
            }
            Some(b'/') if self.in_tag => {
                self.expect_ascii(b'>')?;
                self.selfclose = true;
                Ok(Some(XmlToken::TagOpenEnd))
            }
            Some(_) if self.in_tag => {
                self.n -= 1;
                let attr_key = self.consume_identifier()?;
                self.expect_ascii(b'=')?;
                self.expect_ascii(b'"')?;
                let attr_value = self.consume_until('"')?;
                self.expect_ascii(b'"')?;
                Ok(Some(XmlToken::Attribute(attr_key, XmlStr::new(attr_value))))
            }
            Some(_) => {
                self.n -= 1;
                Ok(Some(XmlToken::Text(XmlStr::new(self.consume_until('<')?))))
            }
            None => Ok(None),
        }
    }

    pub fn tag_open_start(&mut self, expect: &str) -> Result<(), XmlError> {
        match self.next_token()?.ok_or(XmlError::UnexpectedEof)? {
            XmlToken::TagOpenStart(tag) if tag == expect => Ok(()),
            XmlToken::TagOpenStart(_) => Err(XmlError::NameMismatch),
            _ => Err(XmlError::UnexpectedToken),
        }
    }

    pub fn tag_open_end(&mut self) -> Result<(), XmlError> {
        match self.next_token()?.ok_or(XmlError::UnexpectedEof)? {
            XmlToken::TagOpenEnd => Ok(()),
            _ => Err(XmlError::UnexpectedToken),
        }
    }

    pub fn tag_close(&mut self, expect: &str) -> Result<(), XmlError> {
        match self.next_token()?.ok_or(XmlError::UnexpectedEof)? {
            XmlToken::TagClose(tag) if tag == expect => Ok(()),
            XmlToken::TagClose(tag) if tag.is_empty() => Ok(()),
            XmlToken::TagClose(_) if expect.is_empty() => Ok(()),
            XmlToken::TagClose(_) => Err(XmlError::NameMismatch),
            _ => Err(XmlError::UnexpectedToken),
        }
    }

    pub fn text(&mut self) -> Result<XmlStr<'a>, XmlError> {
        match self.next_token()?.ok_or(XmlError::UnexpectedEof)? {
            XmlToken::Text(s) => Ok(s),
            _ => Err(XmlError::UnexpectedToken),
        }
    }

    pub fn attr(&mut self) -> Result<(&'a str, XmlStr<'a>), XmlError> {
        match self.next_token()?.ok_or(XmlError::UnexpectedEof)? {
            XmlToken::Attribute(key, value) => Ok((key, value)),
            _ => Err(XmlError::UnexpectedToken),
        }
    }

    pub fn attr_or_tag_open_end(&mut self) -> Result<Result<(&'a str, XmlStr<'a>), ()>, XmlError> {
        match self.next_token()?.ok_or(XmlError::UnexpectedEof)? {
            XmlToken::Attribute(key, value) => Ok(Ok((key, value))),
            XmlToken::TagOpenEnd => Ok(Err(())),
            _ => Err(XmlError::UnexpectedToken),
        }
    }

    pub fn tag_open_or_close(
        &mut self,
        expect_close: &str,
    ) -> Result<Result<&'a str, ()>, XmlError> {
        match self.next_token()?.ok_or(XmlError::UnexpectedEof)? {
            XmlToken::TagOpenStart(tag) => Ok(Ok(tag)),
            XmlToken::TagClose(tag) if tag == expect_close => Ok(Err(())),
            XmlToken::TagClose(tag) if tag.is_empty() => Ok(Err(())),
            XmlToken::TagClose(_) if expect_close.is_empty() => Ok(Err(())),
            XmlToken::TagClose(_) => Err(XmlError::NameMismatch),
            _ => Err(XmlError::UnexpectedToken),
        }
    }

    pub fn check_end(&mut self) -> Result<(), XmlError> {
        match self.next_token()? {
            Some(_) => Err(XmlError::TrailingChars),
            None => Ok(()),
        }
    }

    fn consume_ascii(&mut self) -> Option<u8> {
        let c = *self.s.as_bytes()[self.n..].first()?;
        self.n += 1;
        Some(c)
    }

    fn opt_consume_ascii(&mut self, expect: u8) -> Option<()> {
        let c = *self.s.as_bytes()[self.n..].first()?;
        if c == expect {
            self.n += 1;
            Some(())
        } else {
            None
        }
    }

    fn expect_ascii(&mut self, expect: u8) -> Result<(), XmlError> {
        let c = *self.s.as_bytes()[self.n..]
            .first()
            .ok_or(XmlError::UnexpectedEof)?;
        if c == expect {
            self.n += 1;
            Ok(())
        } else {
            Err(XmlError::UnexpectedChar)
        }
    }

    fn consume_identifier(&mut self) -> Result<&'a str, XmlError> {
        let bytes = self.s.as_bytes();
        let start = self.n;
        loop {
            if self.n >= self.s.len() {
                break;
            }
            match bytes[self.n] {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b':' => self.n += 1,
                _ => break,
            }
        }
        if self.n == start {
            return Err(XmlError::InvalidIdentifier);
        }
        Ok(&self.s[start..self.n])
    }

    fn consume_until(&mut self, expect: char) -> Result<&'a str, XmlError> {
        let mut i = self.n;
        while i < self.s.len() {
            let c = self.s[i..].chars().next().unwrap();
            if c == expect {
                let result = &self.s[self.n..i];
                self.n = i;
                return Ok(result);
            }
            i += c.len_utf8();
        }
        Err(XmlError::UnexpectedEof)
    }

    fn consume_whitespace(&mut self) {
        let bytes = self.s.as_bytes();
        while self.n < bytes.len() {
            if bytes[self.n].is_ascii_whitespace() {
                self.n += 1;
            } else {
                return;
            }
        }
    }
}

#[derive(Debug)]
pub enum XmlToken<'a> {
    TagOpenStart(&'a str),
    TagOpenEnd,
    TagClose(&'a str),
    Attribute(&'a str, XmlStr<'a>),
    Text(XmlStr<'a>),
}

#[derive(Debug)]
pub struct XmlStr<'a> {
    pub(crate) s: &'a str,
}

impl<'a> XmlStr<'a> {
    pub fn raw(&self) -> &'a str {
        self.s
    }

    #[cfg(feature = "alloc")]
    pub fn parsed(&self) -> Cow<'a, str> {
        let mut i = 0;
        while i < self.s.len() {
            let s = &self.s[i..];
            let c = s.chars().next().unwrap();
            if c == '&' {
                if let Some((c, mut rest)) = starts_with_xml_escape_code(s) {
                    let mut ret = String::from(&self.s[0..i]);
                    ret.push(c);
                    let mut i = 0;
                    while i < rest.len() {
                        let s = &rest[i..];
                        let c = s.chars().next().unwrap();
                        if c == '&' {
                            let (c, new_rest) =
                                starts_with_xml_escape_code(s).unwrap_or(('&', &s[1..]));
                            ret.push(c);
                            rest = new_rest;
                            i = 0;
                        } else {
                            ret.push(c);
                            i += c.len_utf8();
                        }
                    }
                    return Cow::Owned(ret);
                }
            }
            i += c.len_utf8();
        }
        Cow::Borrowed(self.s)
    }

    #[cfg(feature = "heapless")]
    pub fn heapless<const N: usize>(&self) -> Result<heapless::String<N>, ()> {
        if self.s.len() > N {
            return Err(());
        }
        let mut ret = heapless::String::new();
        let mut i = 0;
        let mut rest = self.s;
        while i < rest.len() {
            let s = &rest[i..];
            let c = s.chars().next().unwrap();
            if c == '&' {
                let (c, new_rest) = starts_with_xml_escape_code(s).unwrap_or(('&', &s[1..]));
                ret.push(c)?;
                rest = new_rest;
                i = 0;
            } else {
                ret.push(c)?;
                i += c.len_utf8();
            }
        }
        Ok(ret)
    }

    fn new(s: &'a str) -> Self {
        Self { s }
    }
}

#[derive(Debug)]
pub enum XmlError {
    UnexpectedChar,
    InvalidIdentifier,
    NameMismatch,
    UnexpectedToken,
    UnexpectedEof,
    TrailingChars,
    InvalidField,
    InvalidVariant,
    InvalidValue,
    DuplicateField,
    MissingField,
}

fn skip_xml_header(s: &str) -> Result<&str, XmlError> {
    let bytes = s.as_bytes();
    if &bytes[0..5] != b"<?xml" {
        return Ok(s);
    }
    let mut n = 5;
    loop {
        if n >= bytes.len() {
            return Err(XmlError::UnexpectedEof);
        }
        match bytes[n] {
            b'?' => match bytes.get(n + 1) {
                Some(b'>') => return Ok(&s[(n + 2)..]),
                Some(_) => return Err(XmlError::UnexpectedChar),
                None => return Err(XmlError::UnexpectedEof),
            },
            _ => n += 1,
        }
    }
}

#[cfg(feature = "alloc")]
fn starts_with_xml_escape_code(s: &str) -> Option<(char, &str)> {
    if let Some(rest) = s[1..].strip_prefix("lt;") {
        Some(('<', rest))
    } else if let Some(rest) = s[1..].strip_prefix("gt;") {
        Some(('>', rest))
    } else if let Some(rest) = s[1..].strip_prefix("amp;") {
        Some(('&', rest))
    } else if let Some(rest) = s[1..].strip_prefix("quot;") {
        Some(('"', rest))
    } else if let Some(rest) = s[1..].strip_prefix("apos;") {
        Some(('\'', rest))
    } else {
        None
    }
}
