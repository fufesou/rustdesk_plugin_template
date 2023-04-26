use plugin_common::{lazy_static::lazy_static, libc, CbLog, ResultType};
use std::{
    ffi::{c_char, c_void, CStr},
    sync::{Arc, Mutex},
};

pub mod init;
pub mod call;
pub mod desc;
pub mod errno;
pub mod handler;

/// Callback to send message to peer or ui.
/// peer, target, id are utf8 strings(null terminated).
///
/// peer:    The peer id.
/// target:  "peer", "ui", "conf".
/// id:      The id of this plugin.
/// content: The content.
/// len:     The length of the content.
type CbMsg = extern "C" fn(
    peer: *const c_char,
    target: *const c_char,
    id: *const c_char,
    content: *const c_void,
    len: usize,
);
/// Get local peer id.
///
/// The returned string is utf8 string(null terminated) and must be freed by caller.
type CbGetId = extern "C" fn() -> *const c_char;
/// Callback to get the config.
/// peer, key are utf8 strings(null terminated).
///
/// peer: The peer id.
/// id:   The id of this plugin
/// key:  The key of the config.
///
/// The returned string is utf8 string(null terminated) and must be freed by caller.
type CbGetConf =
    extern "C" fn(peer: *const c_char, id: *const c_char, key: *const c_char) -> *const c_char;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Callbacks {
    pub msg: CbMsg,
    pub get_conf: CbGetConf,
    pub get_id: CbGetId,
    pub log: CbLog,
}

#[inline]
pub fn cstr_to_string(cstr: *const c_char) -> ResultType<String> {
    Ok(String::from_utf8(unsafe {
        CStr::from_ptr(cstr).to_bytes().to_vec()
    })?)
}

#[inline]
pub fn str_to_cstr(s: &str, out: *mut *mut c_char, out_buf_len: *mut usize) {
    let s = s.as_bytes();
    unsafe {
        *out = libc::malloc(s.len()) as *mut c_char;
        *out_buf_len = s.len();
        libc::memcpy(
            *out as *mut libc::c_void,
            s.as_ptr() as *const libc::c_void,
            s.len(),
        );
    }
}

#[inline]
pub fn str_to_cstr_ret(s: &str) -> *const c_char {
    let mut s = s.as_bytes().to_vec();
    s.push(0);
    unsafe {
        let r = libc::malloc(s.len()) as *mut c_char;
        libc::memcpy(
            r as *mut libc::c_void,
            s.as_ptr() as *const libc::c_void,
            s.len(),
        );
        r
    }
}

#[inline]
pub fn make_return_code_msg(code: i32, msg: &str) -> *const c_void {
    let mut out = code.to_le_bytes().to_vec();
    out.extend(msg.as_bytes());
    out.push(0);
    unsafe {
        let r = libc::malloc(out.len()) as *mut c_char;
        libc::memcpy(
            r as *mut libc::c_void,
            out.as_ptr() as *const libc::c_void,
            out.len(),
        );
        r as *const c_void
    }
}
