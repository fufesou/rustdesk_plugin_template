use crate::*;
use std::ffi::{c_char, c_void};

lazy_static! {
    pub(crate) static ref INIT_DATA: Arc<Mutex<Option<InitData>>> = Default::default();
}

#[repr(C)]
pub struct InitData {
    pub version: *const c_char,
    pub cbs: Callbacks,
}

unsafe impl Send for InitData {}

impl Clone for InitData {
    fn clone(&self) -> Self {
        unsafe {
            let version = libc::malloc(libc::strlen(self.version) + 1) as *mut c_char;
            libc::memcpy(
                version as *mut libc::c_void,
                self.version as *const libc::c_void,
                libc::strlen(self.version) + 1,
            );
            let cbs = self.cbs;
            InitData { version, cbs }
        }
    }
}

impl Drop for InitData {
    fn drop(&mut self) {
        unsafe {
            if !self.version.is_null() {
                libc::free(self.version as *mut c_void);
            }
        }
    }
}

pub fn init(handler: Box<dyn handler::Handler>, desc: desc::Desc, info: *const InitData) -> *const c_void {
    let ret = set_init_data(info);
    if !ret.is_null() {
        return ret;
    }
    handler::set_handler(handler);
    desc::set_desc(desc);
    plugin_common::plog::set_log(INIT_DATA.lock().unwrap().as_ref().unwrap().cbs.log);
    std::ptr::null()
}

pub fn reset(info: *const InitData) -> *const c_void {
    let ret = set_init_data(info);
    if !ret.is_null() {
        return ret;
    }
    std::ptr::null()
}

pub fn clear() -> *const c_void {
    *INIT_DATA.lock().unwrap() = None;
    std::ptr::null()
}

fn set_init_data(info: *const InitData) -> *const c_void {
    unsafe {
        if info.is_null() || (*info).version.is_null() {
            return make_return_code_msg(
                crate::errno::ERR_PLUGIN_MSG_INIT_INVALID,
                "Invalid InitData, null pointer",
            );
        }
        *INIT_DATA.lock().unwrap() = Some((*info).clone());
    }
    std::ptr::null()
}
