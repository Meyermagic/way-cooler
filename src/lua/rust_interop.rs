//! Rust code which is called from lua in the init file
#![deny(dead_code)]

use std::ops::Deref;

use hlua::{self, Lua, LuaTable};
use hlua::any::AnyLuaValue;

use registry::{self, RegistryError, AccessFlags};
use commands;
use keys::{self, KeyPress, KeyEvent};
use convert::json::{json_to_lua, lua_to_json};

use super::thread::{RUNNING, ERR_LOCK_RUNNING};

type ValueResult = Result<AnyLuaValue, &'static str>;

/// We've `include!`d the code which initializes from the Lua side.

/// Register all the Rust functions for the lua libraries
pub fn register_libraries(lua: &mut Lua) {
    trace!("Registering Rust libraries...");
    {
        let mut rust_table: LuaTable<_> = lua.empty_array("__rust");
        rust_table.set("init_workspaces", hlua::function1(init_workspaces));
        rust_table.set("register_lua_key", hlua::function2(register_lua_key));
        rust_table.set("register_command_key", hlua::function3(register_command_key));
        rust_table.set("register_mouse_modifier", hlua::function1(register_mouse_modifier));
        rust_table.set("keypress_index", hlua::function1(keypress_index));
        rust_table.set("ipc_run", hlua::function1(ipc_run));
        rust_table.set("ipc_get", hlua::function1(ipc_get));
        rust_table.set("ipc_set", hlua::function2(ipc_set));
    }
    trace!("Executing Lua init...");
    let init_code = include_str!("../../lib/lua/lua_init.lua");
    let _: () = lua.execute::<_>(init_code)
        .expect("Unable to execute Lua init code!");
    trace!("Lua register_libraries complete");
}

/// Run a command
fn ipc_run(command: String) -> Result<(), &'static str> {
    commands::get(&command).map(|com| com())
        .ok_or("Command does not exist")
}

/// IPC 'get' handler
fn ipc_get(key: String) -> ValueResult {
    match registry::get_data(&key) {
        Ok(regdata) => {
            let (flags, arc_data) = regdata.resolve();
            if flags.contains(AccessFlags::READ()) {
                Ok(json_to_lua(arc_data.deref().clone()))
            } else {
                Err("Cannot read that key")
            }
        },
        Err(err) => match err {
            RegistryError::InvalidOperation =>
                Err("Cannot get that key, use set or assign"),
            RegistryError::KeyNotFound =>
                Err("Key not found")
        }
    }
}

/// ipc 'set' handler
fn ipc_set(key: String, value: AnyLuaValue) -> Result<(), &'static str> {
    let json = try!(lua_to_json(value)
                    .map_err(|_| "Unable to convert value to JSON!"));
    match *RUNNING.read().expect(ERR_LOCK_RUNNING) {
        true => {
            registry::set_json(key.clone(), json.clone())
        },
        false => {
            registry::set_json_ignore_flags(key.clone(), json.clone())
        }
    }.map(|data| data.call(json.clone()))
        .or_else(|err| {
            match err {
                RegistryError::InvalidOperation => {
                    Err("That value can not be set!")
                },
                RegistryError::KeyNotFound => {
                    registry::insert_json(key, AccessFlags::READ(), json.clone());
                    Ok(())
                }
            }
        })
}

fn init_workspaces(_options: AnyLuaValue) -> Result<(), &'static str> {
    error!("Attempting to call `init_workspaces`, this is not implemented");
    Ok(())
}

/// Registers a modifier to be used in conjunction with mouse commands
fn register_mouse_modifier(modifier: String) -> Result<(), String> {
    let modifier = try!(keys::keymod_from_names(&[modifier.as_str()]));
    keys::register_mouse_modifier(modifier);
    Ok(())
}

/// Registers a command keybinding.
fn register_command_key(mods: String, command: String, _repeat: bool) -> Result<(), String> {
    if let Ok(press) = keypress_from_string(&mods) {
        commands::get(&command)
            .ok_or(format!("Command {} for keybinding {} not found", command, press))
            .map(|command| { keys::register(press, KeyEvent::Command(command)); })
    }
    else {
        Err(format!("Invalid keypress {}, {}", mods, command))
    }
}

/// Rust half of registering a Lua key: store the KeyPress in the keys table
/// and send Lua back the index for __key_map.
fn register_lua_key(mods: String, _repeat: bool) -> Result<String, String> {
    keypress_from_string(&mods)
        .map(|press| {
            keys::register(press.clone(), KeyEvent::Lua);
            press.get_lua_index_string()
        }).map_err(|_| format!("Invalid keys '{}'", mods))}

/// Parses a keypress from a string
fn keypress_from_string(mods: &str) -> Result<KeyPress, String> {
    let parts: Vec<&str> = mods.split(',').collect();
    if let Some((ref key, mods)) = parts.split_last() {
        KeyPress::from_key_names(mods, &key)
    }
    else {
        Err(format!("Invalid key '{}'", mods))
    }
}

fn keypress_index(press: String) -> Result<String, String> {
    keypress_from_string(&press).map(|key| key.get_lua_index_string())
}
