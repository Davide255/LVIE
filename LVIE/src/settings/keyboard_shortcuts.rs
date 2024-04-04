use serde::{Deserialize, Serialize};

pub const DEFAULT: &str = r#"<Keyboard>
    <key value="o">
        <binding action="openfile">
            <ctrl/>
        </binding>
    </key>
    <key value="r">
        <binding action="rotate-90-deg">
            <ctrl/><shift/>
        </binding>
    </key>
</Keyboard>"#;

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub enum MODIFIER {
    #[serde(rename = "alt")]
    ALT,
    #[serde(rename = "ctrl")]
    CTRL,
    #[serde(rename = "shift")]
    SHIFT
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Binding {
    #[serde(rename = "@action")]
    action: String,
    #[serde(rename = "$value")]
    modifiers: Vec<MODIFIER>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Key {
    #[serde(rename = "@value")]
    value: String,
    #[serde(rename = "binding")]
    bindings: Vec<Binding>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Keyboard {
    #[serde(rename = "key")]
    keys: Vec<Key>
}

impl IntoIterator for Keyboard {
    type Item = Key;
    type IntoIter = std::vec::IntoIter<Key>;

    fn into_iter(self) -> Self::IntoIter {
        self.keys.into_iter()
    }
}

pub fn load_from_file(fd: Option<String>) -> std::io::Result<Result<Keyboard, quick_xml::DeError>> {
    let f = std::fs::read_to_string({
        if fd.is_none() {
            String::from(".LVIE/keyboard_shortcuts.xml")
        } else {
            fd.unwrap()
        }
    });

    if f.is_err() {
        return Err(f.unwrap_err());
    } else {
        return Ok(load_from_xml(f.unwrap()));
    }
}

pub fn load_from_xml(content: String) -> Result<Keyboard, quick_xml::DeError> {
    quick_xml::de::from_str(&content)
}

fn _prettify_xml(content: &mut String) {
    *content = content.replace("<key", "\n\t<key");
    *content = content.replace("<binding", "\n\t\t<binding");
    *content = content.replace("</key>", "\n\t</key>\n");
}
