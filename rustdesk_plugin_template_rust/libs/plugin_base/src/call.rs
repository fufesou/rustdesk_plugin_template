use crate::{
    cstr_to_string,
    desc::{self, get_desc},
    early_return_value,
    errno::*,
    handler::*,
    init::INIT_DATA,
    make_return_code_msg,
};
use plugin_common::{libc, serde_json};
use std::ffi::{c_char, c_void};

macro_rules! early_call_return_if_true {
    ($e:expr, $code: ident, $($arg:tt)*) => {
        if $e {
            return make_return_code_msg($code, &format_args!($($arg)*).to_string());
        }
    };
}

macro_rules! early_return_if_true {
    ($e:expr, $code: ident, $($arg:tt)*) => {
        if $e {
            return HandlerRet {
                code: $code,
                msg: format_args!($($arg)*).to_string(),
                msgs: Msgs::default(),
            };
        }
    };
}

#[inline]
fn is_method(method: *const c_char, target: &[u8]) -> bool {
    target == unsafe { std::slice::from_raw_parts(method as *const u8, target.len()) }
}

fn process_return(plugin_id: &str, peer: String, ret: HandlerRet) -> *const c_void {
    for msg in ret.msgs.to_config.into_iter() {
        call_msg_cb(
            peer.clone(),
            MSG_TO_CONFIG_TARGET,
            plugin_id.to_owned(),
            msg.as_bytes(),
        );
    }

    for msg in ret.msgs.to_peer.into_iter() {
        call_msg_cb(
            peer.clone(),
            MSG_TO_PEER_TARGET,
            plugin_id.to_owned(),
            msg.as_bytes(),
        );
    }

    for msg in ret.msgs.to_ui.into_iter() {
        let mut content = MSG_TO_UI_FLUTTER_CHANNEL_REMOTE.to_le_bytes().to_vec();
        content.extend(serde_json::to_string(&msg).unwrap().as_bytes());
        call_msg_cb(
            peer.clone(),
            MSG_TO_UI_TARGET,
            plugin_id.to_owned(),
            &content,
        );
    }

    match ret.code {
        ERR_SUCCESS => std::ptr::null(),
        _ => make_return_code_msg(ret.code, &ret.msg),
    }
}

pub fn plugin_call(
    method: *const c_char,
    peer: *const c_char,
    args: *const c_void,
    len: usize,
    out: *mut *mut c_void,
    out_len: *mut usize,
) -> *const c_void {
    early_call_return_if_true!(
        INIT_DATA.lock().unwrap().is_none(),
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
            return make_return_code_msg(
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
    } else if is_method(method, METHOD_HANDLE_CONN) {
        handle_msg_conn(d, args, len)
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

    let local_peer_id = if let Some(data) = INIT_DATA.lock().unwrap().as_ref() {
        let id_ptr = (data.cbs.get_id)();
        let local_peer_id = early_return_value!(
            cstr_to_string(id_ptr),
            ERR_PLUGIN_MSG_GET_LOCAL_PEER_ID,
            "parse local peer id",
        );
        unsafe {
            libc::free(id_ptr as _);
        }
        local_peer_id
    } else {
        return HandlerRet {
            code: ERR_PLUGIN_MSG_INIT,
            msg: "Callbacks must be set before calling any other functions".to_owned(),
            msgs: Msgs::default(),
        };
    };
    (*get_handler().as_ref().unwrap()).handle_ui_event(d, local_peer_id, msg_ui)
}

fn handle_msg_conn(d: &desc::Desc, args: *const c_void, _len: usize) -> HandlerRet {
    let msgs = Msgs::default();
    let msg_peer = early_return_value!(
        MsgPeer::from_c_str(args as _),
        ERR_CALL_INVALID_ARGS,
        "parse args"
    );

    early_return_if_true!(
        msg_peer.id != d.id,
        ERR_PEER_ID_MISMATCH,
        "Id mismatch {}",
        msg_peer.id
    );
    HandlerRet {
        code: ERR_CALL_UNIMPLEMENTED,
        msg: format!("Unimplemented"),
        msgs,
    }
}

fn handle_msg_peer(
    d: &desc::Desc,
    args: *const c_void,
    _len: usize,
    out: *mut *mut c_void,
    out_len: *mut usize,
) -> HandlerRet {
    let msg_peer = early_return_value!(
        MsgPeer::from_c_str(args as _),
        ERR_CALL_INVALID_ARGS,
        "parse args"
    );

    early_return_if_true!(
        msg_peer.id != d.id,
        ERR_PEER_ID_MISMATCH,
        "Id mismatch {}",
        msg_peer.id
    );

    if !out.is_null() {
        (*get_handler().as_ref().unwrap()).handle_client_event(d, msg_peer, out, out_len)
    } else {
        (*get_handler().as_ref().unwrap()).handle_server_event(d, msg_peer)
    }
}

fn call_msg_cb(mut peer: String, target: &[u8], mut id: String, content: &[u8]) {
    if let Some(data) = INIT_DATA.lock().unwrap().as_ref() {
        peer.push('\0');
        id.push('\0');
        (data.cbs.msg)(
            peer.as_ptr() as _,
            target.as_ptr() as _,
            id.as_ptr() as _,
            content.as_ptr() as _,
            content.len() as _,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_ui_to_string() {
        let msg = MsgToConfig::new_string(
            "desc id".to_owned(),
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
