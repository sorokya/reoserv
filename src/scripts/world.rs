use mlua::{Function, LuaSerdeExt, Table, UserData, UserDataMethods, Value};

use crate::{world::WorldHandle, ITEM_DB};

#[derive(Clone)]
pub struct World {
    world: WorldHandle,
}

impl World {
    pub fn new(world: WorldHandle) -> Self {
        Self { world }
    }
}

impl UserData for World {
    fn add_methods<T: UserDataMethods<Self>>(methods: &mut T) {
        methods.add_method("quake", |_, this, strength: i32| {
            this.world.quake(strength);
            Ok(())
        });

        methods.add_method("on_tick", |lua, _, callback: Function| {
            let globals = lua.globals();

            let script_name: String = globals
                .get("_SCRIPT_NAME")
                .map_err(|_| mlua::Error::RuntimeError("Script name not found".into()))?;

            let events: Table = globals
                .get("events")
                .unwrap_or_else(|_| lua.create_table().unwrap());

            let tick_table: Table = events
                .get("on_tick")
                .unwrap_or_else(|_| lua.create_table().unwrap());

            let script_table: Table = tick_table
                .get(script_name.to_string())
                .unwrap_or_else(|_| lua.create_table().unwrap());

            let callback_id = script_table.len().unwrap_or(0) + 1;
            script_table.set(callback_id, callback)?;

            tick_table.set(script_name.to_string(), script_table)?;
            events.set("on_tick", tick_table)?;
            globals.set("events", events)?;

            Ok(())
        });

        methods.add_method("get_item_by_name", |lua, _, name: String| {
            match ITEM_DB
                .items
                .iter()
                .find(|item| item.name.to_lowercase() == name.to_lowercase())
            {
                Some(item) => match lua.to_value(item) {
                    Ok(item) => Ok(item),
                    Err(e) => {
                        error!("Failed to serialize item: {}", e);
                        Err(e)
                    }
                },
                None => Ok(Value::Nil),
            }
        });

        methods.add_method(
            "on_player_command",
            |lua, _, (cmd, callback): (String, Function)| {
                let globals = lua.globals();

                let script_name: String = globals
                    .get("_SCRIPT_NAME")
                    .map_err(|_| mlua::Error::RuntimeError("Script name not found".into()))?;

                let events: Table = globals
                    .get("events")
                    .unwrap_or_else(|_| lua.create_table().unwrap());

                let command_table: Table = events
                    .get("on_player_command")
                    .unwrap_or_else(|_| lua.create_table().unwrap());

                let script_table: Table = command_table
                    .get(script_name.to_string())
                    .unwrap_or_else(|_| lua.create_table().unwrap());

                script_table.set(cmd, callback)?;

                command_table.set(script_name.to_string(), script_table)?;
                events.set("on_player_command", command_table)?;
                globals.set("events", events)?;
                Ok(())
            },
        );
    }
}
