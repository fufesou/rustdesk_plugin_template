use plugin_base::desc::*;
use plugin_common::serde_json;
use std::collections::HashMap;

pub const ID: &str = "TemplateTestIdRust";
pub const NAME: &str = "RustDesk Plugin Template";
pub const VERSION: &str = "v0.1.0";
pub const UI_HOST_MAIN_LOCATION: &str = "host|main|settings|plugin";
pub const UI_HOST_MAIN_KEY: &str = "allow-opt";
pub const UI_CLIENT_REMOTE_LOCATION: &str = "client|remote|toolbar|display";
pub const UI_CLIENT_REMOTE_KEY: &str = "peer-opt";

pub fn get_desc() -> Desc {
    let mut desc = Desc {
        id: ID.to_string(),
        name: NAME.to_string(),
        version: VERSION.to_string(),
        author: "RustDesk".to_string(),
        home: "https://rustdesk.com".to_string(),
        license: "MIT".to_string(),
        published: "2020-02-03 13:05:02".to_string(),
        released: "2023-02-03 13:05:02".to_string(),
        github: "https://github/demo".to_string(),
        ..Default::default()
    };

    desc.location = Location {
        ui: {
            let mut ui = HashMap::new();
            ui.insert(
                UI_HOST_MAIN_LOCATION.to_string(),
                UiType::Checkbox(UiCheckbox {
                    key: UI_HOST_MAIN_KEY.to_string(),
                    text: "Allow option".to_string(),
                    tooltip: "".to_string(),
                    action: "".to_string(),
                }),
            );
            ui.insert(
                UI_CLIENT_REMOTE_LOCATION.to_string(),
                UiType::Checkbox(UiCheckbox {
                    key: UI_CLIENT_REMOTE_KEY.to_string(),
                    text: "Option to peer".to_string(),
                    tooltip: "".to_string(),
                    action: "".to_string(),
                }),
            );
            ui
        },
    };

    desc.config = Config {
        shared: vec![ConfigItem {
            key: UI_HOST_MAIN_KEY.to_string(),
            default: CONFIG_VALUE_FALSE.to_string(),
            description: "Allow option".to_string(),
        }],
        peer: vec![ConfigItem {
            key: UI_CLIENT_REMOTE_KEY.to_string(),
            default: CONFIG_VALUE_FALSE.to_string(),
            description: "Trigger option on peer side".to_string(),
        }],
    };
    desc
}

#[inline]
pub fn get_desc_string() -> String {
    serde_json::to_string(&get_desc()).unwrap()
}
