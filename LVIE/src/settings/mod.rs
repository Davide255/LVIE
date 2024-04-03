pub mod history;
pub mod ketboard_shortcuts;

mod xml {
    extern crate indexmap;
    use indexmap::IndexMap;
    use std::fmt;
    use std::io::{self, Write};

    /// Represents an XML element
    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct XMLElement {
        name: String,
        attributes: IndexMap<String, String>,
        content: XMLElementContent,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    enum XMLElementContent {
        Empty,
        Elements(Vec<XMLElement>),
        Text(String),
    }

    impl fmt::Display for XMLElement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let mut s: Vec<u8> = Vec::new();
            self.write(&mut s)
                .expect("Failure writing output to Vec<u8>");
            write!(f, "{}", unsafe { String::from_utf8_unchecked(s) })
        }
    }

    impl XMLElement {
        /// Creates a new empty XML element using the given name for the tag.
        pub fn new(name: impl ToString) -> Self {
            XMLElement {
                name: name.to_string(),
                attributes: IndexMap::new(),
                content: XMLElementContent::Empty,
            }
        }

        /// Adds an attribute to the XML element. The attribute value can take any
        /// type which implements [`fmt::Display`].
        pub fn add_attribute(&mut self, name: impl ToString, value: impl ToString) {
            self.attributes
                .insert(name.to_string(), escape_str(&value.to_string()));
        }

        /// Adds a child element to the XML element.
        /// The new child will be placed after previously added children.
        ///
        /// This method may only be called on an element that has children or is
        /// empty.
        ///
        /// # Panics
        ///
        /// Panics if the element contains text.
        pub fn add_child(&mut self, child: XMLElement) {
            use XMLElementContent::*;
            match self.content {
                Empty => {
                    self.content = Elements(vec![child]);
                }
                Elements(ref mut list) => {
                    list.push(child);
                }
                Text(_) => {
                    panic!("Attempted adding child element to element with text.");
                }
            }
        }

        /// Adds text to the XML element.
        ///
        /// This method may only be called on an empty element.
        ///
        /// # Panics
        ///
        /// Panics if the element is not empty.
        pub fn add_text(&mut self, text: impl ToString) {
            use XMLElementContent::*;
            match self.content {
                Empty => {
                    self.content = Text(escape_str(&text.to_string()));
                }
                _ => {
                    panic!("Attempted adding text to non-empty element.");
                }
            }
        }

        /// Outputs a UTF-8 XML document, where this element is the root element.
        ///
        /// Output is properly indented.
        ///
        /// # Errors
        ///
        /// Returns Errors from writing to the Write object.
        pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
            writeln!(writer, r#"<?xml version = "1.0" encoding = "UTF-8"?>"#)?;
            self.write_level(&mut writer, 0)
        }

        fn write_level<W: Write>(&self, writer: &mut W, level: usize) -> io::Result<()> {
            use XMLElementContent::*;
            let prefix = "\t".repeat(level);
            match &self.content {
                Empty => {
                    writeln!(
                        writer,
                        "{}<{}{} />",
                        prefix,
                        self.name,
                        self.attribute_string()
                    )?;
                }
                Elements(list) => {
                    writeln!(
                        writer,
                        "{}<{}{}>",
                        prefix,
                        self.name,
                        self.attribute_string()
                    )?;
                    for elem in list {
                        elem.write_level(writer, level + 1)?;
                    }
                    writeln!(writer, "{}</{}>", prefix, self.name)?;
                }
                Text(text) => {
                    writeln!(
                        writer,
                        "{}<{}{}>{}</{1}>",
                        prefix,
                        self.name,
                        self.attribute_string(),
                        text
                    )?;
                }
            }
            Ok(())
        }

        fn attribute_string(&self) -> String {
            if self.attributes.is_empty() {
                "".to_owned()
            } else {
                let mut result = "".to_owned();
                for (k, v) in &self.attributes {
                    result = result + &format!(r#" {}="{}""#, k, v);
                }
                result
            }
        }
    }

    fn escape_str(input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }
}

pub struct Settings {
    pub backend: crate::core::RenderingBackends,
}

pub fn load_settings() -> Settings {
    Settings {
        backend: crate::core::RenderingBackends::GPU,
    }
}

pub fn first_start() {
    let mut dir = std::path::Path::new(file!());

    println!("{}", dir.display());
}