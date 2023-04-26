use std::ffi::c_char;

/// Callback to log.
///
/// level, msg are utf8 strings(null terminated).
/// level: "error", "warn", "info", "debug", "trace".
/// msg:   The message.
pub type CbLog = extern "C" fn(level: *const c_char, msg: *const c_char);

pub const __LOG_LEVEL_TRACE: &[u8; 6] = b"trace\0";
pub const __LOG_LEVEL_DEBUG: &[u8; 6] = b"debug\0";
pub const __LOG_LEVEL_INFO: &[u8; 5] = b"info\0";
pub const __LOG_LEVEL_WARN: &[u8; 5] = b"warn\0";
pub const __LOG_LEVEL_ERROR: &[u8; 6] = b"error\0";

static mut LOG_CB: Option<CbLog> = None;

pub fn set_log(cb: CbLog) {
    unsafe {
        LOG_CB = Some(cb);
    }
}

// WARNING: this is not part of the crate's public API and is subject to change at any time
pub fn __get_log() -> &'static Option<CbLog> {
    unsafe { &LOG_CB }
}

#[macro_export]
macro_rules! log_level {
    ($level: ident, $lvl:ident, $($arg:tt)*) => {
        match $crate::plog::__get_log() {
            Some(cb) => {
                let mut s = format!($($arg)*);
                s.push('\0');
                cb(
                    $crate::plog::$lvl.as_ptr() as _,
                    s.as_ptr() as _,
                );
            },
            None => log::$level!($($arg)*),
        }
    };
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        $crate::log_level!(trace, __LOG_LEVEL_TRACE, $($arg)*);
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::log_level!(debug, __LOG_LEVEL_DEBUG, $($arg)*);
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::log_level!(info, __LOG_LEVEL_INFO, $($arg)*);
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::log_level!(warn, __LOG_LEVEL_WARN, $($arg)*);
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::log_level!(error, __LOG_LEVEL_ERROR, $($arg)*);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_default() {
        trace!("trace");
        debug!("debug");
        info!("info");
        warn!("warn");
        error!("error");
    }
    #[no_mangle]
    pub extern "C" fn _log_cb(level: *const i8, msg: *const i8) {
        let level = unsafe { std::ffi::CStr::from_ptr(level).to_str().unwrap() };
        let msg = unsafe { std::ffi::CStr::from_ptr(msg).to_str().unwrap() };
        println!("{}: {}", level, msg);
    }

    #[test]
    fn test_log_custom_print() {
        set_log(_log_cb);
        trace!("trace");
        debug!("debug");
        info!("info");
        warn!("warn");
        error!("error");
    }
}
