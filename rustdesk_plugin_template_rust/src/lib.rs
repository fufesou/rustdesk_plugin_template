mod api;
mod call;
mod desc;

#[cfg(test)]
mod tests {
    use dlopen::symbor::Library;
    use plugin_base::{desc::Desc, init::InitData, str_to_cstr_ret, Callbacks, PluginReturn};
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
                                plugin_common::debug!("{} method found {}", path, stringify!($field));
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
                    let mut init_ret = (self.init)(data as _);
                    if !init_ret.is_success() {
                        let (code, msg) = init_ret.get_code_msg();
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
                    let mut clear_ret = (self.clear)();
                    if !clear_ret.is_success() {
                        let (code, msg) = clear_ret.get_code_msg();
                        plugin_common::error!(
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
        init: extern "C" fn(*const InitData) -> PluginReturn,
        reset: extern "C" fn() -> PluginReturn,
        clear: extern "C" fn() -> PluginReturn,
        desc: extern "C" fn() -> *const c_void,
        client_call:
            extern "C" fn(
                method: *const c_char,
                peer: *const c_char,
                args: *const c_void,
                len: usize,
            ) -> PluginReturn,
        server_call:
            extern "C" fn(
                method: *const c_char,
                peer: *const c_char,
                args: *const c_void,
                len: usize,
                out: *mut *mut c_void,
                out_len: *mut usize,
            ) -> PluginReturn
    );

    #[no_mangle]
    extern "C" fn msg(
        _peer: *const c_char,
        _target: *const c_char,
        _id: *const c_char,
        _content: *const c_void,
        _len: usize,
    ) -> PluginReturn {
        println!("msg called");
        PluginReturn::success()
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
    extern "C" fn native(
        _method: *const c_char,
        _json: *const c_char,
        _raw: *const c_void,
        _raw_len: usize,
    ) -> plugin_base::NativeReturnValue {
        plugin_base::NativeReturnValue {
            return_type: 0,
            data: std::ptr::null_mut(),
        }
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
                native,
            },
        };
        plugin.init(&init_data, &path).unwrap();
        let args_content = crate::call::PluginPeerMsg::new_string("local peer id".to_owned());
        let mut args = super::call::MsgPeer::new_string(
            &super::desc::get_desc(),
            "on".to_owned(),
            args_content,
        );
        args.push('\0');
        let mut out = std::ptr::null_mut();
        let mut out_len: usize = 0;
        let mut ret = (plugin.server_call)(
            "handle_peer\0".as_ptr() as _,
            "remote peer id\0".as_ptr() as _,
            args.as_bytes().as_ptr() as _,
            args.as_bytes().len(),
            &mut out,
            &mut out_len,
        );
        if ret.is_success() {
            println!("call success");
        } else {
            let (code, msg) = ret.get_code_msg();
            println!("code: {}, msg: {}", code, msg);
        }

        std::thread::sleep(std::time::Duration::from_secs(3));
    }
}
