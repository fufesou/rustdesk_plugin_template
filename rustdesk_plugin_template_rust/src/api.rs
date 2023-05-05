use plugin_base::{init::InitData, str_to_cstr_ret, PluginReturn};
use std::ffi::{c_char, c_void};
use std::ptr::null_mut;

extern "C" {}

#[no_mangle]
pub extern "C" fn init(info: *const InitData) -> PluginReturn {
    plugin_base::init::init(
        Box::new(super::call::HandlerTemplate {}),
        super::desc::get_desc(),
        info,
    )
}

#[no_mangle]
pub extern "C" fn reset(info: *const InitData) -> PluginReturn {
    plugin_base::init::reset(info)
}

#[no_mangle]
pub extern "C" fn clear() -> PluginReturn {
    plugin_base::init::clear()
}

#[no_mangle]
pub extern "C" fn desc() -> *const c_char {
    str_to_cstr_ret(&super::desc::get_desc_string())
}

#[no_mangle]
pub extern "C" fn client_call(
    method: *const c_char,
    peer: *const c_char,
    args: *const c_void,
    len: usize,
) -> PluginReturn {
    plugin_base::call::plugin_call(method, peer, args, len, null_mut(), null_mut())
}

#[no_mangle]
pub extern "C" fn server_call(
    method: *const c_char,
    peer: *const c_char,
    args: *const c_void,
    len: usize,
    out: *mut *mut c_void,
    out_len: *mut usize,
) -> PluginReturn {
    plugin_base::call::plugin_call(method, peer, args, len, out, out_len)
}
