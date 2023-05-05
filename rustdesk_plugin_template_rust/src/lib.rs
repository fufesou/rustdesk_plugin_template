use std::ffi::{c_char, c_int};
use std::ptr::null;

#[repr(C)]
#[derive(Debug)]
pub struct PluginReturn {
    pub code: c_int,
    pub msg: *const c_char,
}

impl PluginReturn {
    pub fn success() -> Self {
        Self {
            code: 0,
            msg: null(),
        }
    }
}

#[no_mangle]
pub fn init() -> PluginReturn {
    let r = PluginReturn::success();
    println!("REMOVE ME ==================== init return {:?}", &r);
    r
}

#[cfg(test)]
mod tests {
    use crate::PluginReturn;
    use dlopen::symbor::Library;

    #[allow(dead_code)]
    struct Plugin {
        _lib: Library,
        id: Option<String>,
        path: String,
        init: extern "C" fn() -> PluginReturn,
    }

    impl Plugin {
        fn new(path: &str) -> Self {
            let lib = match Library::open(path) {
                Ok(lib) => lib,
                Err(e) => {
                    panic!("Failed to load library {}, {}", path, e);
                }
            };

            let init = match unsafe { lib.symbol::<extern "C" fn() -> PluginReturn>("init") } {
                Ok(m) => {
                    log::debug!("init method found {}", path);
                    *m
                }
                Err(e) => {
                    panic!("Failed to load init func {}, {}", path, e);
                }
            };

            Self {
                _lib: lib,
                id: None,
                path: path.to_string(),
                init,
            }
        }

        fn init(&self) {
            let ret = (self.init)();
            if ret.code != 0 {
                println!("REMOVE ME ==================== {:?}", &ret);
                panic!("Init does not return success")
            }
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
        let plugin = Plugin::new(&path);
        plugin.init();
    }
}
