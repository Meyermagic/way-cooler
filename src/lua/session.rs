//! Lua session object which runs a Lua session in its own thread.

use std::sync::mpsc::Receiver;

use hlua::Lua;

use lua::message::LuaMessage;

//// Structure used to
pub struct LuaSession<'a> {
    receiver: Receiver<LuaMessage>,
    lua: Lua<'a>
}
