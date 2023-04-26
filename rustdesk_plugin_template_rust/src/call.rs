use super::desc;
use plugin_base::{
    desc::{Desc, CONFIG_VALUE_FALSE, CONFIG_VALUE_TRUE},
    early_return_value,
    errno::*,
    handler::*,
};
use plugin_common::{
    log,
    serde_derive::{Deserialize, Serialize},
    serde_json,
};
use std::ffi::c_void;

const MSG_PEER_METHOD_TURN_ON: &str = "on";
const MSG_PEER_METHOD_TURN_OFF: &str = "off";
const MSG_PEER_METHOD_NOTIFY_TURN_ON: &str = "notify_on";
const MSG_PEER_METHOD_NOTIFY_TURN_OFF: &str = "notify_off";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PluginPeerMsg {
    f1: String,
}

impl PluginPeerMsg {
    pub(crate) fn new_string(f1: String) -> String {
        serde_json::to_string(&PluginPeerMsg { f1 }).unwrap()
    }
}

pub struct HandlerTemplate;

impl Handler for HandlerTemplate {
    fn handle_ui_event(&self, d: &Desc, local_peer_id: String, msg_ui: MsgFromUi) -> HandlerRet {
        let mut ret = HandlerRet::default();
        match &msg_ui.location as _ {
            desc::UI_CLIENT_REMOTE_LOCATION => match &msg_ui.key as _ {
                desc::UI_CLIENT_REMOTE_KEY => match &msg_ui.value as _ {
                    CONFIG_VALUE_TRUE | CONFIG_VALUE_FALSE => {
                        ret.code = ERR_SUCCESS;
                        ret.msg = "".to_string();

                        let turn_on_off = if &msg_ui.value == CONFIG_VALUE_FALSE {
                            MSG_PEER_METHOD_TURN_OFF
                        } else {
                            MSG_PEER_METHOD_TURN_ON
                        };
                        let msg_peer_content = PluginPeerMsg::new_string(local_peer_id);
                        ret.msgs.to_peer.push(MsgPeer::new_string(
                            d,
                            turn_on_off.to_string(),
                            msg_peer_content,
                        ));
                    }
                    _ => {}
                },
                _ => {}
            },
            desc::UI_HOST_MAIN_LOCATION => match &msg_ui.key as _ {
                desc::UI_HOST_MAIN_KEY => match &msg_ui.value as _ {
                    CONFIG_VALUE_TRUE | CONFIG_VALUE_FALSE => {
                        ret.code = ERR_SUCCESS;
                        ret.msg = "".to_string();
                        ret.msgs.to_config.push(MsgToConfig::new_string(
                            d.id.clone(),
                            CONFIG_TYPE_SHARED.to_string(),
                            desc::UI_HOST_MAIN_KEY.to_owned(),
                            msg_ui.value,
                            Some(ConfigToUi {
                                channel: MSG_TO_UI_FLUTTER_CHANNEL_MAIN,
                                location: desc::UI_HOST_MAIN_LOCATION.to_owned(),
                            }),
                        ));
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }

        ret
    }

    fn handle_client_event(
        &self,
        d: &Desc,
        msg_peer: MsgPeer,
        out: *mut *mut c_void,
        out_len: *mut usize,
    ) -> HandlerRet {
        let mut ret = HandlerRet::default();
        match &msg_peer.method as &str {
            MSG_PEER_METHOD_TURN_ON => {
                let args = early_return_value!(
                    serde_json::from_str::<PluginPeerMsg>(&msg_peer.content as _),
                    ERR_CALL_INVALID_ARGS,
                    "parse msg content {}",
                    msg_peer.content
                );
                // process on event
                println!("Plugin: process event {:?}", &args);
                ret.code = ERR_SUCCESS;
                ret.msg = "".to_owned();
                // ret.code = EER_CALL_FAILED;
                // ret.msg = "something error".to_string();
                MsgPeer::fill_out(
                    d,
                    MSG_PEER_METHOD_NOTIFY_TURN_ON.to_string(),
                    ret.msg.clone(),
                    out,
                    out_len,
                );
            }
            MSG_PEER_METHOD_TURN_OFF => {
                let args = early_return_value!(
                    serde_json::from_str::<PluginPeerMsg>(&msg_peer.content as _),
                    ERR_CALL_INVALID_ARGS,
                    "parse msg content {}",
                    msg_peer.content
                );
                // process on event
                println!("Plugin: process event {:?}", &args);
                ret.code = ERR_SUCCESS;
                ret.msg = "".to_owned();
                // ret.code = EER_CALL_FAILED;
                // ret.msg = "something error".to_string();
                MsgPeer::fill_out(
                    d,
                    MSG_PEER_METHOD_NOTIFY_TURN_OFF.to_string(),
                    ret.msg.clone(),
                    out,
                    out_len,
                );
            }
            _ => {
                ret.code = ERR_CALL_INVALID_ARGS;
                ret.msg = format!("Invalid method {}", msg_peer.method);
            }
        }
        ret
    }

    fn handle_server_event(&self, d: &Desc, msg_peer: MsgPeer) -> HandlerRet {
        let mut ret = HandlerRet::default();
        match &msg_peer.method as &str {
            MSG_PEER_METHOD_NOTIFY_TURN_ON => {
                if msg_peer.content.is_empty() {
                    plugin_common::debug!("Plugin: turn on succeeded");
                    ret.code = ERR_SUCCESS;
                    // to-do: translate
                    ret.msg = "success".to_owned();
                    ret.msgs
                        .to_config
                        .push(Self::make_msg_to_config(&d.id, CONFIG_VALUE_TRUE));
                    ret.msgs.to_ui.push(Self::make_msg_to_msgbox("on"));
                } else {
                    plugin_common::debug!("Plugin: turn on failed, {}", &msg_peer.content);
                    ret.code = ERR_PEER_ON_FAILED;
                    // to-do: translate
                    ret.msg = format!("{} {}", "Failed to turn on", msg_peer.content);
                    ret.msgs
                        .to_config
                        .push(Self::make_msg_to_config(&d.id, CONFIG_VALUE_FALSE));
                    ret.msgs
                        .to_ui
                        .push(Self::make_msg_to_msgbox("Failed to turn on"));
                }
            }
            MSG_PEER_METHOD_NOTIFY_TURN_OFF => {
                if msg_peer.content.is_empty() {
                    plugin_common::debug!("Plugin: turn off succeeded");
                    ret.code = ERR_SUCCESS;
                    // to-do: translate
                    ret.msg = "success".to_owned();
                    ret.msgs
                        .to_config
                        .push(Self::make_msg_to_config(&d.id, CONFIG_VALUE_FALSE));
                    ret.msgs.to_ui.push(Self::make_msg_to_msgbox("off"));
                } else {
                    plugin_common::debug!("Plugin: turn off failed, {}", &msg_peer.content);
                    ret.code = ERR_PEER_OFF_FAILED;
                    // to-do: translate
                    ret.msg = format!("{} {}", "Failed to turn off", msg_peer.content);
                    ret.msgs
                        .to_config
                        .push(Self::make_msg_to_config(&d.id, CONFIG_VALUE_TRUE));
                    ret.msgs
                        .to_ui
                        .push(Self::make_msg_to_msgbox("Failed to turn off"));
                }
            }
            _ => {
                ret.code = ERR_CALL_INVALID_ARGS;
                ret.msg = format!("Invalid method {}", msg_peer.method);
            }
        }
        ret
    }
}

impl HandlerTemplate {
    #[inline]
    fn make_msg_to_config(id: &str, v: &str) -> String {
        MsgToConfig::new_string(
            id.to_string(),
            CONFIG_TYPE_PEER.to_string(),
            desc::UI_CLIENT_REMOTE_KEY.to_owned(),
            v.to_owned(),
            Some(ConfigToUi {
                channel: MSG_TO_UI_FLUTTER_CHANNEL_REMOTE,
                location: desc::UI_CLIENT_REMOTE_LOCATION.to_owned(),
            }),
        )
    }

    #[inline]
    fn make_msg_to_msgbox(msg: &str) -> MsgToUi {
        MsgToUi::new_msg_msgbox("custom-nocancel", "Plugin title", msg, "")
    }
}
