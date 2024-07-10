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
    SHIFT,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Binding {
    #[serde(rename = "@action")]
    action: String,
    #[serde(rename = "$value")]
    modifiers: Vec<MODIFIER>,
}

impl Binding {
    pub fn action(&self) -> &String {
        &self.action
    }

    pub fn modifiers(&self) -> &Vec<MODIFIER> {
        &self.modifiers
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Key {
    #[serde(rename = "@value")]
    value: String,
    #[serde(rename = "binding")]
    bindings: Vec<Binding>,
}

impl Key {
    pub fn is(&self, x: &String) -> bool {
        x == &self.value
    }

    pub fn value(&self) -> &String {
        &self.value
    }

    pub fn bindings(&self) -> &Vec<Binding> {
        &self.bindings
    }

    pub fn get_binding_by_modifiers(&self, modifiers: &Vec<MODIFIER>) -> Option<&Binding> {
        for k in &self.bindings {
            if k.modifiers == *modifiers {
                return Some(k);
            }
        }
        None
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Keyboard {
    #[serde(rename = "key")]
    keys: Vec<Key>,
}

impl IntoIterator for Keyboard {
    type Item = Key;
    type IntoIter = std::vec::IntoIter<Key>;

    fn into_iter(self) -> Self::IntoIter {
        self.keys.into_iter()
    }
}

impl<'a> IntoIterator for &'a Keyboard {
    type Item = &'a Key;
    type IntoIter = std::slice::Iter<'a, Key>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.keys).into_iter()
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

pub fn prettify_keyboard_xml(content: String) -> String {
    return content
        .replace("<key", "\n\t<key")
        .replace("<binding", "\n\t\t<binding")
        .replace("</key>", "\n\t</key>\n");
}

#[macro_export]
macro_rules! build_shortcuts {
    ( $editor:literal $( >$option:literal $( --$suboption:literal : $function:tt )* )* ) => {
        fn handle_shortcut_action(ww: Weak<LVIE>, action: Vec<&str>) {
            match action[1] {
                $($option => {
                    match action[2] {
                        $(
                            $suboption => $function(ww.unwrap(), &action[2..]),
                        )*
                        _ => ()
                    }
                })*
                _ => ()
            }
        }
    };
}

impl Default for Keyboard {
    fn default() -> Self {
        Keyboard {
            keys: vec![
                Key {
                    value: String::from("o"),
                    bindings: vec![Binding {
                        action: "editor.file.open".into(),
                        modifiers: vec![MODIFIER::CTRL],
                    }],
                },
                Key {
                    value: String::from("e"),
                    bindings: vec![Binding {
                        action: "editor.file.close".into(),
                        modifiers: vec![MODIFIER::CTRL, MODIFIER::SHIFT],
                    }],
                },
                Key {
                    value: String::from("r"),
                    bindings: vec![Binding {
                        action: "editor.image.rotate-90-deg".into(),
                        modifiers: vec![MODIFIER::CTRL, MODIFIER::SHIFT],
                    }],
                },
            ],
        }
    }
}
