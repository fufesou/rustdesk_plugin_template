use plugin_common::serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

pub const CONFIG_VALUE_TRUE: &str = "1";
pub const CONFIG_VALUE_FALSE: &str = "0";

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct UiButton {
    pub key: String,
    pub text: String,
    pub icon: String, // icon can be int in flutter, but string in other ui framework.
    pub tooltip: String,
    pub action: String, // The action to be triggered when the button is clicked.
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct UiCheckbox {
    pub key: String,
    pub text: String,
    pub tooltip: String,
    pub action: String, // The action to be triggered when the checkbox is checked or unchecked.
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
pub enum UiType {
    Button(UiButton),
    Checkbox(UiCheckbox),
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Location {
    pub ui: HashMap<String, UiType>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ConfigItem {
    pub key: String,
    pub default: String,
    pub description: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Config {
    pub shared: Vec<ConfigItem>,
    pub peer: Vec<ConfigItem>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Desc {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub home: String,
    pub license: String,
    pub published: String,
    pub released: String,
    pub github: String,
    pub location: Location,
    pub config: Config,
}

static mut DESC: Option<Desc> = None;

pub(crate) fn set_desc(desc: Desc) {
    unsafe {
        DESC = Some(desc);
    }
}

pub(crate) fn get_desc() -> &'static Option<Desc> {
    unsafe { &DESC }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plugin_common::serde_json;

    #[test]
    fn test_ui_to_string() {
        let ui = UiType::Button(UiButton {
            key: "key".to_string(),
            text: "text".to_string(),
            icon: "icon".to_string(),
            tooltip: "tooltip".to_string(),
            action: "action".to_string(),
        });
        println!("ui button: {}", serde_json::to_string(&ui).unwrap());
        let ui = UiType::Checkbox(UiCheckbox {
            key: "key".to_string(),
            text: "text".to_string(),
            tooltip: "tooltip".to_string(),
            action: "action".to_string(),
        });
        println!("ui checkbox: {}", serde_json::to_string(&ui).unwrap());
    }
}
