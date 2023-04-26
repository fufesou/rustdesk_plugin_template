mod api;
mod call;
mod desc;

#[cfg(test)]
mod tests {
    use dlopen::symbor::Library;
    use plugin_base::{desc::Desc, handler, init::InitData, str_to_cstr_ret, Callbacks};
    use plugin_common::{bail, libc, log, serde_json, ResultType};
    use std::ffi::{c_char, c_void, CStr};

    #[inline]
    fn free_c_ptr(ret: *mut c_void) {
        if !ret.is_null() {
            unsafe {
                libc::free(ret);
            }
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

    macro_rules! make_plugin {
        ($($field:ident : $tp:ty),+) => {
            #[allow(dead_code)]
            struct Plugin {
                _lib: Library,
                id: Option<String>,
                path: String,
                $($field: $tp),+
            }

            impl Plugin {
                fn new(path: &str) -> ResultType<Self> {
                    let lib = match Library::open(path) {
                        Ok(lib) => lib,
                        Err(e) => {
                            bail!("Failed to load library {}, {}", path, e);
                        }
                    };

                    $(let $field = match unsafe { lib.symbol::<$tp>(stringify!($field)) } {
                            Ok(m) => {
                                log::debug!("{} method found {}", path, stringify!($field));
                                *m
                            },
                            Err(e) => {
                                bail!("Failed to load {} func {}, {}", path, stringify!($field), e);
                            }
                        }
                    ;)+

                    Ok(Self {
                        _lib: lib,
                        id: None,
                        path: path.to_string(),
                        $( $field ),+
                    })
                }

                fn desc(&self) -> ResultType<Desc> {
                    let desc_ret = (self.desc)();
                    let s = unsafe { CStr::from_ptr(desc_ret as _) };
                    let desc = serde_json::from_str(s.to_str()?);
                    free_c_ptr(desc_ret as _);
                    Ok(desc?)
                }

                fn init(&self, data: &InitData, path: &str) -> ResultType<()> {
                    let init_ret = (self.init)(data as _);
                    if !init_ret.is_null() {
                        let (code, msg) = get_code_msg_from_ret(init_ret);
                        free_c_ptr(init_ret as _);
                        bail!(
                            "Failed to init plugin {}, code: {}, msg: {}",
                            path,
                            code,
                            msg
                        );
                    }
                    Ok(())
                }

                fn clear(&self, id: &str) {
                    let clear_ret = (self.clear)();
                    if !clear_ret.is_null() {
                        let (code, msg) = get_code_msg_from_ret(clear_ret);
                        free_c_ptr(clear_ret as _);
                        log::error!(
                            "Failed to clear plugin {}, code: {}, msg: {}",
                            id,
                            code,
                            msg
                        );
                    }
                }
            }

            impl Drop for Plugin {
                fn drop(&mut self) {
                    let id = self.id.as_ref().unwrap_or(&self.path);
                    self.clear(id);
                }
            }
        }
    }

    make_plugin!(
        init: extern "C" fn(*const InitData) -> *const c_void,
        reset: extern "C" fn() -> *const c_void,
        clear: extern "C" fn() -> *const c_void,
        desc: extern "C" fn() -> *const c_void,
        client_call:
            extern "C" fn(
                method: *const c_char,
                peer: *const c_char,
                args: *const c_void,
                len: usize,
            ) -> *const c_void,
        server_call:
            extern "C" fn(
                method: *const c_char,
                peer: *const c_char,
                args: *const c_void,
                len: usize,
                out: *mut *mut c_void,
                out_len: *mut usize,
            ) -> *const c_void
    );

    #[no_mangle]
    extern "C" fn msg(
        _peer: *const c_char,
        _target: *const c_char,
        _id: *const c_char,
        _content: *const c_void,
        _len: usize,
    ) {
        println!("msg called");
    }

    #[no_mangle]
    extern "C" fn get_conf(
        _peer: *const c_char,
        _id: *const c_char,
        _key: *const c_char,
    ) -> *const c_char {
        println!("get_conf called");
        std::ptr::null()
    }

    #[no_mangle]
    extern "C" fn get_id() -> *const c_char {
        println!("get_id called");
        str_to_cstr_ret("test id")
    }

    #[no_mangle]
    extern "C" fn log_cb(level: *const i8, msg: *const i8) {
        let level = unsafe { std::ffi::CStr::from_ptr(level).to_str().unwrap() };
        let msg = unsafe { std::ffi::CStr::from_ptr(msg).to_str().unwrap() };
        println!("{}: {}", level, msg);
    }

    #[test]
    fn test_plugin() {
        #[cfg(target_os = "windows")]
        let lib_ext = "dll";
        #[cfg(target_os = "linux")]
        let lib_ext = "so";
        #[cfg(target_os = "macos")]
        let lib_ext = "dylib";
        let path = format!("target/debug/plugin_template.{}", lib_ext);
        let plugin = Plugin::new(&path).unwrap();
        let _desc = plugin.desc().unwrap();
        // println!("{:?}", desc);
        let init_data = InitData {
            version: str_to_cstr_ret("test version"),
            cbs: Callbacks {
                msg,
                get_conf,
                get_id,
                log: log_cb,
            },
        };
        plugin.init(&init_data, &path).unwrap();
        let args_content = crate::call::PluginPeerMsg::new_string("local peer id".to_owned());
        let mut args =
            handler::MsgPeer::new_string(&super::desc::get_desc(), "on".to_owned(), args_content);
        args.push('\0');
        let mut out = std::ptr::null_mut();
        let mut out_len: usize = 0;
        let ret = (plugin.server_call)(
            "handle_peer\0".as_ptr() as _,
            "remote peer id\0".as_ptr() as _,
            args.as_bytes().as_ptr() as _,
            args.as_bytes().len(),
            &mut out,
            &mut out_len,
        );
        if ret.is_null() {
            println!("call success");
        } else {
            let (code, msg) = get_code_msg_from_ret(ret);
            println!("code: {}, msg: {}", code, msg);
            free_c_ptr(ret as _);
        }

        std::thread::sleep(std::time::Duration::from_secs(3));
    }
}
