slint::include_modules!();
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use slint::Weak;

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

impl Binding {
    pub fn action(&self) -> String {
        self.action.clone()
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
    bindings: Vec<Binding>
}

impl Key {
    pub fn is(&self, x: &String) -> bool{
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

fn _prettify_xml(content: &mut String) {
    *content = content.replace("<key", "\n\t<key");
    *content = content.replace("<binding", "\n\t\t<binding");
    *content = content.replace("</key>", "\n\t</key>\n");
}


struct EditorActions {
    options: HashMap<&'static str, HashMap<&'static str, Box<dyn FnOnce(Weak<LVIE>, &[&str]) -> ()>>>
}

impl EditorActions {
    pub fn add_fn(&mut self, first: &'static str, second: &'static str, f: Box<dyn FnOnce(Weak<LVIE>, &[&str]) + 'static>) {
        if self.options.get(&first).is_some() {
            self.options.get_mut(first).insert(&mut HashMap::from([(second, f)]));
        } else {
            let mut inner = HashMap::new();
            inner.insert(second, f);
            self.options.insert(first, inner);
        }
    }
}
//const Editor: HashMap<&'static str, HashMap<&'static str, Box<dyn Fn(Weak<LVIE>, &[&str]) -> ()>>> = HashMap::from(
//    [
//        ("file", HashMap::from(
//            [
//                ("open", Box::new(|ww: Weak<LVIE>, args: &[&str]| {
//                        ww.upgrade_in_event_loop(|Window| Window.global::<ToolbarCallbacks>().invoke_open_file());
//                    }))
//            ])
//        )
//    ]
//);
//