use crate::{desc::Desc, errno::*};
use plugin_common::{
    serde_derive::{Deserialize, Serialize},
    serde_json, ResultType,
};
use std::ffi::{c_char, c_void, CStr};

pub const MSG_TO_UI_FLUTTER_CHANNEL_MAIN: u16 = 0x01 << 0;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub const MSG_TO_UI_FLUTTER_CHANNEL_CM: u16 = 0x01 << 1;
#[cfg(any(target_os = "android", target_os = "ios"))]
pub const MSG_TO_UI_FLUTTER_CHANNEL_CM: u16 = 0x01;
pub const MSG_TO_UI_FLUTTER_CHANNEL_REMOTE: u16 = 0x01 << 2;
pub const MSG_TO_UI_FLUTTER_CHANNEL_TRANSFER: u16 = 0x01 << 3;
pub const MSG_TO_UI_FLUTTER_CHANNEL_FORWARD: u16 = 0x01 << 4;

pub const METHOD_HANDLE_UI: &[u8; 10] = b"handle_ui\0";
pub const METHOD_HANDLE_PEER: &[u8; 12] = b"handle_peer\0";
pub const METHOD_HANDLE_LISTEN_EVENT: &[u8; 20] = b"handle_listen_event\0";
pub const EVENT_ON_CONN_CLIENT: &str = "on_conn_client";
pub const EVENT_ON_CONN_SERVER: &str = "on_conn_server";
pub const EVENT_ON_CONN_CLOSE_CLIENT: &str = "on_conn_close_client";
pub const EVENT_ON_CONN_CLOSE_SERVER: &str = "on_conn_close_server";
pub const MSG_TO_PEER_TARGET: &[u8; 5] = b"peer\0";
pub const MSG_TO_UI_TARGET: &[u8; 3] = b"ui\0";
pub const MSG_TO_CONFIG_TARGET: &[u8; 7] = b"config\0";
pub const MSG_TO_EXT_SUPPORT_TARGET: &[u8; 12] = b"ext-support\0";
pub const CONFIG_TYPE_SHARED: &str = "shared";
pub const CONFIG_TYPE_PEER: &str = "peer";

#[macro_export]
macro_rules! early_return_value {
    ($e:expr, $code: ident, $($arg:tt)*) => {
        match $e {
            Err(e) => {
                return $crate::handler::HandlerRet {
                    code: $code,
                    msg: format!("Failed to {} '{:?}'", format_args!($($arg)*), e),
                    msgs: $crate::handler::Msgs::default(),
                };
            }
            Ok(v) => v,
        }
    };
}

#[macro_export]
macro_rules! early_return_if_true {
    ($e:expr, $code: ident, $($arg:tt)*) => {
        if $e {
            return $crate::handler::HandlerRet {
                code: $code,
                msg: format_args!($($arg)*).to_string(),
                msgs: $crate::handler::Msgs::default(),
            };
        }
    };
}

#[derive(Serialize)]
pub struct MsgToUiMsgBox {
    pub r#type: String,
    pub title: String,
    pub text: String,
    pub link: String,
}

#[derive(Serialize)]
#[serde(tag = "t", content = "c")]
pub enum MsgToUi {
    MsgBox(MsgToUiMsgBox),
}

impl MsgToUi {
    pub fn new_msg_msgbox(r#type: &str, title: &str, text: &str, link: &str) -> Self {
        MsgToUi::MsgBox(MsgToUiMsgBox {
            r#type: r#type.to_owned(),
            title: title.to_owned(),
            text: text.to_owned(),
            link: link.to_owned(),
        })
    }
}

#[derive(Serialize)]
pub struct ConfigToUi {
    pub channel: u16,
    pub location: String,
}

#[derive(Serialize)]
pub struct MsgToConfig {
    pub r#type: String,
    pub key: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui: Option<ConfigToUi>, // If not None, send msg to ui.
}

impl MsgToConfig {
    fn new(r#type: String, key: String, value: String, ui: Option<ConfigToUi>) -> Self {
        MsgToConfig {
            r#type,
            key,
            value,
            ui,
        }
    }

    pub fn new_string(
        r#type: String,
        key: String,
        value: String,
        ui: Option<ConfigToUi>,
    ) -> String {
        serde_json::to_string(&MsgToConfig::new(r#type, key, value, ui)).unwrap()
    }
}

#[derive(Debug, Serialize)]
pub struct MsgToExtSupport {
    pub r#type: String,
    pub data: Vec<u8>,
}

#[derive(Default)]
pub struct Msgs {
    pub to_ui: Vec<MsgToUi>,
    pub to_config: Vec<String>,
    pub to_peer: Vec<String>,
}

#[derive(Deserialize)]
pub struct MsgFromUi {
    pub id: String,
    pub name: String,
    pub location: String,
    pub key: String,
    pub value: String,
    pub action: String,
}

#[derive(Deserialize)]
pub struct MsgListenEvent {
    pub event: String,
}

impl MsgListenEvent {
    #[inline]
    pub fn from_cstr(cstr: *const c_char) -> ResultType<Self> {
        Ok(serde_json::from_str(unsafe {
            CStr::from_ptr(cstr).to_str()?
        })?)
    }
}

pub struct HandlerRet {
    pub code: i32,
    pub msg: String,
    pub msgs: Msgs,
}

impl Default for HandlerRet {
    fn default() -> Self {
        Self {
            code: ERR_CALL_INVALID_ARGS,
            msg: format!("Default return msg"),
            msgs: Msgs::default(),
        }
    }
}

static mut PLUGIN_HANDLER: Option<Box<dyn Handler>> = None;

pub trait Handler {
    fn handle_ui_event(&self, d: &Desc, local_peer_id: String, msg_ui: MsgFromUi) -> HandlerRet;
    fn handle_client_event(
        &self,
        d: &Desc,
        args: *const c_void,
        len: usize,
        out: *mut *mut c_void,
        out_len: *mut usize,
    ) -> HandlerRet;
    fn handle_server_event(&self, d: &Desc, args: *const c_void, len: usize) -> HandlerRet;
    fn handle_listen_event(
        &self,
        d: &Desc,
        local_peer_id: String,
        remote_peer_id: &str,
        event: MsgListenEvent,
    ) -> HandlerRet;
}

pub fn set_handler(handler: Box<dyn Handler>) {
    unsafe {
        PLUGIN_HANDLER = Some(handler);
    }
}

pub fn get_handler() -> &'static Option<Box<dyn Handler>> {
    unsafe { &PLUGIN_HANDLER }
}
