pub use anyhow::{self, bail};
pub use lazy_static;
pub use libc;
pub use log;
pub use serde_derive;
pub use serde_json;

pub mod plog;
pub use plog::CbLog;

pub type ResultType<F, E = anyhow::Error> = anyhow::Result<F, E>;
