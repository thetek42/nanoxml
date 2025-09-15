use core::fmt::Arguments;
use core::fmt::Result as FmtResult;
use core::fmt::Write;

pub struct XmlBuilder<'w, W: Write> {
    pub(crate) writer: &'w mut W,
}

impl<'w, W: Write> XmlBuilder<'w, W> {
    pub fn new(writer: &'w mut W) -> Self {
        Self { writer }
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

    pub fn attr_start(&mut self, key: &str) -> FmtResult {
        write!(self.writer, " {key}=\"")
    }

    pub fn attr_end(&mut self) -> FmtResult {
        self.writer.write_char('"')
    }

    pub fn attr(&mut self, key: &str, value: &str) -> FmtResult {
        self.attr_start(key)?;
        self.text(value)?;
        self.attr_end()
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
            self.attr(attr.0, attr.1)?;
        }
        self.tag_open_end()
    }
}

impl<W: Write> Write for XmlBuilder<'_, W> {
    fn write_str(&mut self, s: &str) -> FmtResult {
        self.writer.write_str(s)
    }

    fn write_char(&mut self, c: char) -> FmtResult {
        self.writer.write_char(c)
    }

    fn write_fmt(&mut self, args: Arguments<'_>) -> FmtResult {
        self.writer.write_fmt(args)
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
