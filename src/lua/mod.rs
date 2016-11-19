//! Lua functionality

#[cfg(test)]
mod tests;

mod message;
mod thread;
mod rust_interop;
mod init_path;
mod session;

pub use self::message::{LuaQuery, LuaFunc, LuaResponse};
pub use self::thread::{init, running, send, LuaSendError};
pub use self::session::LuaSession;

pub const DEFAULT_INIT_FILE: &'static str = include_str!("../../config/init.lua");
