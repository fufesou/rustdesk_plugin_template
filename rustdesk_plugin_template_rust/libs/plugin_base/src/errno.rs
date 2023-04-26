#![allow(dead_code)]

pub const ERR_SUCCESS: i32 = 0;

// ======================================================
// errors that will be handled by RustDesk

// not loaded
pub const ERR_PLUGIN_LOAD: i32 = 10001;
// not initialized
pub const ERR_PLUGIN_MSG_INIT: i32 = 10101;
pub const ERR_PLUGIN_MSG_INIT_INVALID: i32 = 10102;
pub const ERR_PLUGIN_MSG_GET_LOCAL_PEER_ID: i32 = 10103;
// invalid
pub const ERR_CALL_UNIMPLEMENTED: i32 = 10201;
pub const ERR_CALL_INVALID_METHOD: i32 = 10202;
pub const ERR_CALL_NOT_SUPPORTED_METHOD: i32 = 10203;
pub const ERR_CALL_INVALID_PEER: i32 = 10204;
// failed on calling
pub const ERR_CALL_INVALID_ARGS: i32 = 10301;
pub const ERR_PEER_ID_MISMATCH: i32 = 10302;

// ======================================================
// errors that should be handled by the plugin

pub const EER_CALL_FAILED: i32 = 20021;
pub const ERR_PEER_ON_FAILED: i32 = 30012;
pub const ERR_PEER_OFF_FAILED: i32 = 30012;
