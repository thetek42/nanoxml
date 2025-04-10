// TODO: derives

use core::fmt::Error as FmtError;
use core::fmt::Result as FmtResult;
use core::fmt::Write;

pub struct XmlBuilder<'w, W: Write> {
    writer: &'w mut W,
}

impl<'w, W: Write> XmlBuilder<'w, W> {
    pub fn new(writer: &'w mut W) -> Self {
        Self { writer }
    }

    pub fn new_with_xml_header(writer: &'w mut W) -> Result<Self, FmtError> {
        writer.write_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
        Ok(Self { writer })
    }

    pub fn tag_open(&mut self, tag: &str) -> FmtResult {
        write!(self.writer, "<{tag}>")
    }

    pub fn tag_open_start(&mut self, tag: &str) -> FmtResult {
        write!(self.writer, "<{tag}")
    }

    pub fn tag_open_end(&mut self) -> FmtResult {
        self.writer.write_char('>')
    }

    pub fn tag_close(&mut self, tag: &str) -> FmtResult {
        write!(self.writer, "</{tag}>")
    }

    pub fn tag_selfclose(&mut self) -> FmtResult {
        self.writer.write_str("/>")
    }

    pub fn tag_empty(&mut self, tag: &str) -> FmtResult {
        write!(self.writer, "<{tag}/>")
    }

    pub fn attribute(&mut self, key: &str, value: &str) -> FmtResult {
        write!(self.writer, " {key}=\"")?;
        write_escaped(self.writer, value)?;
        self.writer.write_char('"')
    }

    pub fn text(&mut self, text: &str) -> FmtResult {
        write_escaped(self.writer, text)
    }

    pub fn tag_with_text(&mut self, tag: &str, text: &str) -> FmtResult {
        self.tag_open(tag)?;
        self.text(text)?;
        self.tag_close(tag)
    }

    pub fn tag_open_attrs(&mut self, tag: &str, attrs: &[(&str, &str)]) -> FmtResult {
        self.tag_open_start(tag)?;
        for attr in attrs {
            self.attribute(attr.0, attr.1)?;
        }
        self.tag_open_end()
    }
}

fn write_escaped<W: Write>(writer: &mut W, s: &str) -> FmtResult {
    for c in s.chars() {
        match c {
            '<' => writer.write_str("&lt;")?,
            '>' => writer.write_str("&gt;")?,
            '&' => writer.write_str("&amp;")?,
            '"' => writer.write_str("&quot;")?,
            '\'' => writer.write_str("&apos;")?,
            c => writer.write_char(c)?,
        }
    }
    Ok(())
}
