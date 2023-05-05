use crate::{
    cstr_to_string,
    desc::{self, get_desc},
    early_return_if_true, early_return_value,
    errno::*,
    handler::*,
    init::get_init_data,
    PluginReturn,
};
use plugin_common::{libc, serde_json};
use std::ffi::{c_char, c_void};

macro_rules! early_call_return_if_true {
    ($e:expr, $code: ident, $($arg:tt)*) => {
        if $e {
            return PluginReturn::new($code, &format_args!($($arg)*).to_string());
        }
    };
}

#[inline]
fn is_method(method: *const c_char, target: &[u8]) -> bool {
    target == unsafe { std::slice::from_raw_parts(method as *const u8, target.len()) }
}

fn process_return(plugin_id: &str, peer: String, ret: HandlerRet) -> PluginReturn {
    for msg in ret.msgs.to_config.into_iter() {
        let _r = call_msg_cb(
            peer.clone(),
            MSG_TO_CONFIG_TARGET,
            plugin_id.to_owned(),
            msg.as_bytes(),
        );
    }

    for msg in ret.msgs.to_peer.into_iter() {
        let _r = call_msg_cb(
            peer.clone(),
            MSG_TO_PEER_TARGET,
            plugin_id.to_owned(),
            msg.as_bytes(),
        );
    }

    for msg in ret.msgs.to_ui.into_iter() {
        let mut content = MSG_TO_UI_FLUTTER_CHANNEL_REMOTE.to_le_bytes().to_vec();
        content.extend(serde_json::to_string(&msg).unwrap().as_bytes());
        let _r = call_msg_cb(
            peer.clone(),
            MSG_TO_UI_TARGET,
            plugin_id.to_owned(),
            &content,
        );
    }

    match ret.code {
        ERR_SUCCESS => PluginReturn::success(),
        _ => PluginReturn::new(ret.code, &ret.msg),
    }
}

pub fn plugin_call(
    method: *const c_char,
    peer: *const c_char,
    args: *const c_void,
    len: usize,
    out: *mut *mut c_void,
    out_len: *mut usize,
) -> PluginReturn {
    early_call_return_if_true!(
        get_init_data().lock().unwrap().is_none(),
        ERR_PLUGIN_MSG_INIT,
        "Plugin must be initialized before calling any other functions"
    );

    early_call_return_if_true!(
        get_handler().is_none(),
        ERR_PLUGIN_MSG_INIT,
        "Plugin handler must be set before calling any other functions"
    );

    early_call_return_if_true!(
        get_desc().is_none(),
        ERR_PLUGIN_MSG_INIT,
        "Plugin desc must be set before calling any other functions"
    );

    let is_null = method.is_null();
    early_call_return_if_true!(is_null, ERR_CALL_INVALID_METHOD, "method is null");

    let peer = match cstr_to_string(peer) {
        Ok(peer) => peer,
        Err(e) => {
            return PluginReturn::new(
                ERR_CALL_INVALID_PEER,
                &format!("parse remote peer id: {:?}", e),
            )
        }
    };

    let d = get_desc().as_ref().unwrap();

    let ret = if is_method(method, METHOD_HANDLE_UI) {
        handle_msg_ui(d, args, len)
    } else if is_method(method, METHOD_HANDLE_PEER) {
        handle_msg_peer(d, args, len, out, out_len)
    } else if is_method(method, METHOD_HANDLE_LISTEN_EVENT) {
        handle_msg_listen(d, &peer, args, len)
    } else {
        HandlerRet {
            code: ERR_CALL_NOT_SUPPORTED_METHOD,
            msg: format!(
                "Unsupported call of '{:?}'",
                unsafe { std::ffi::CStr::from_ptr(method) }.to_str()
            ),
            msgs: Msgs::default(),
        }
    };

    process_return(&d.id, peer, ret)
}

enum PeerIdOrRet {
    PeerId(String),
    Ret(HandlerRet),
}

fn get_local_peer_id() -> PeerIdOrRet {
    if let Some(data) = get_init_data().lock().unwrap().as_ref() {
        let id_ptr = (data.cbs.get_id)();
        match cstr_to_string(id_ptr) {
            Ok(id) => {
                unsafe {
                    libc::free(id_ptr as _);
                }
                PeerIdOrRet::PeerId(id)
            }
            Err(..) => PeerIdOrRet::Ret(HandlerRet {
                code: ERR_PLUGIN_MSG_GET_LOCAL_PEER_ID,
                msg: "parse local peer id".to_owned(),
                msgs: Msgs::default(),
            }),
        }
    } else {
        PeerIdOrRet::Ret(HandlerRet {
            code: ERR_PLUGIN_MSG_INIT,
            msg: "Callbacks must be set before calling any other functions".to_owned(),
            msgs: Msgs::default(),
        })
    }
}

fn handle_msg_ui(d: &desc::Desc, args: *const c_void, _len: usize) -> HandlerRet {
    let content = early_return_value!(
        cstr_to_string(args as _),
        ERR_CALL_INVALID_ARGS,
        "parse args"
    );
    let msg_ui = early_return_value!(
        serde_json::from_str::<MsgFromUi>(&content as _),
        ERR_CALL_INVALID_ARGS,
        "parse {}",
        content,
    );
    early_return_if_true!(msg_ui.id != d.id, ERR_CALL_INVALID_ARGS, "id mismatch");
    let local_peer_id = match get_local_peer_id() {
        PeerIdOrRet::PeerId(peer_id) => peer_id,
        PeerIdOrRet::Ret(ret) => return ret,
    };
    (*get_handler().as_ref().unwrap()).handle_ui_event(d, local_peer_id, msg_ui)
}

fn handle_msg_listen(
    d: &desc::Desc,
    remote_peer_id: &str,
    args: *const c_void,
    _len: usize,
) -> HandlerRet {
    let event = early_return_value!(
        MsgListenEvent::from_cstr(args as _),
        ERR_CALL_INVALID_ARGS,
        "parse args"
    );
    let local_peer_id = match get_local_peer_id() {
        PeerIdOrRet::PeerId(peer_id) => peer_id,
        PeerIdOrRet::Ret(ret) => return ret,
    };
    (*get_handler().as_ref().unwrap()).handle_listen_event(d, local_peer_id, remote_peer_id, event)
}

fn handle_msg_peer(
    d: &desc::Desc,
    args: *const c_void,
    len: usize,
    out: *mut *mut c_void,
    out_len: *mut usize,
) -> HandlerRet {
    if !out.is_null() {
        (*get_handler().as_ref().unwrap()).handle_client_event(d, args, len, out, out_len)
    } else {
        (*get_handler().as_ref().unwrap()).handle_server_event(d, args, len)
    }
}

pub fn call_msg_cb(
    mut peer: String,
    target: &[u8],
    mut id: String,
    content: &[u8],
) -> (i32, String) {
    if let Some(data) = get_init_data().lock().unwrap().as_ref() {
        peer.push('\0');
        id.push('\0');
        let mut ret = (data.cbs.msg)(
            peer.as_ptr() as _,
            target.as_ptr() as _,
            id.as_ptr() as _,
            content.as_ptr() as _,
            content.len() as _,
        );
        if ret.is_success() {
            (ERR_SUCCESS, "".to_owned())
        } else {
            ret.get_code_msg()
        }
    } else {
        (ERR_SUCCESS, "".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_ui_to_string() {
        let msg = MsgToConfig::new_string(
            CONFIG_TYPE_SHARED.to_owned(),
            "msg key".to_owned(),
            desc::CONFIG_VALUE_TRUE.to_owned(),
            None,
        );
        println!("msg to ui: {}", msg);
        let msg = MsgToUi::new_msg_msgbox("custom-nocancel", "Plugin title", "Failed unknown", "");
        println!("msg to msgbox: {}", serde_json::to_string(&msg).unwrap());
    }
}
