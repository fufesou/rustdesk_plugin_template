use plugin_common::{lazy_static::lazy_static, libc, CbLog, ResultType};
use std::{
    ffi::{c_char, c_int, c_void, CStr},
    sync::{Arc, Mutex},
};

pub mod call;
pub mod desc;
pub mod errno;
pub mod handler;
pub mod init;

/// Callback to send message to peer or ui.
/// peer, target, id are utf8 strings(null terminated).
///
/// peer:    The peer id.
/// target:  "peer", "ui", "conf".
/// id:      The id of this plugin.
/// content: The content.
/// len:     The length of the content.
///
/// Return null ptr if success.
/// Return the error message if failed.  `i32-String` without dash, i32 is a signed little-endian number, the String is utf8 string.
/// The plugin allocate memory with `libc::malloc` and return the pointer.
type CbMsg = extern "C" fn(
    peer: *const c_char,
    target: *const c_char,
    id: *const c_char,
    content: *const c_void,
    len: usize,
) -> *const c_void;
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
/// The native returned value from librustdesk native.
///
/// [Note]
/// The data is owned by librustdesk.
#[repr(C)]
pub struct NativeReturnValue {
    pub return_type: c_int,
    pub data: *const c_void,
}
/// Callback to the librustdesk core.
///
/// method: the method name of this callback.
/// json: the json data for the parameters. The argument *must* be non-null.
/// raw: the binary data for this call, nullable.
/// raw_len: the length of this binary data, only valid when we pass raw data to `raw`.
type CallbackNative = extern "C" fn(
    method: *const c_char,
    json: *const c_char,
    raw: *const c_void,
    raw_len: usize,
) -> NativeReturnValue;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Callbacks {
    pub msg: CbMsg,
    pub get_conf: CbGetConf,
    pub get_id: CbGetId,
    pub log: CbLog,
    // pub native: CallbackNative,
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

#[inline]
fn get_code_msg_from_ret(ret: *const c_void) -> (i32, String) {
    assert!(ret.is_null() == false);
    let code_bytes = unsafe { std::slice::from_raw_parts(ret as *const u8, 4) };
    let code = i32::from_le_bytes([code_bytes[0], code_bytes[1], code_bytes[2], code_bytes[3]]);
    let msg = unsafe { CStr::from_ptr((ret as *const u8).add(4) as _) }
        .to_str()
        .unwrap_or("")
        .to_string();
    (code, msg)
}

#[inline]
fn free_c_ptr(ret: *mut c_void) {
    if !ret.is_null() {
        unsafe {
            libc::free(ret);
        }
    }
}
