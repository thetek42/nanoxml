# nanoxml

A truly minimal XML (de)serializer for Rust.

## Features

- serialization and deserialization of very basic XML (only attributes and elements are supported)
- `derive` for (de)serialization of structs and enums (optional)
- tiny codebase (~400 LoC + optional ~900 LoC for derive)
- no runtime dependencies
- `no_std`
- optional `alloc` support
- optional `defmt` support
- optional `heapless` support
- UTF-8 only

## Derive Attributes

- `#[nanoxml(attr)]`: (de)serialize as attribute (i.e. `key="value"`)
- `#[nanoxml(text)]`: (de)serialize as text (i.e. content between `<tag></tag>` without an additional sub-element for the field)
- `#[nanoxml(rename = "xmlname")]`: use `xmlname` as the attribute key or tag name in the XML
- `#[nanoxml(seq)]`: must be specified for "sequence" fields (e.g. `Vec` or array)
- `#[nanoxml(skip_ser)]`: skip this field when serializing to XML
- `#[nanoxml(default_de)]`: when this field is not present when deserializing the XML, fall back to the `Default::default()` value
- `#[nanoxml(default_de = "func")]`: when this field is not present when deserializing the XML, call `func` to get a fallback value
