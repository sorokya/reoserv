use tealr::{
    mlu::{
        mlua::{self, Function, Table},
        TealData, UserData,
    },
    ToTypename,
};

use crate::world::WorldHandle;

#[derive(Clone, UserData, ToTypename)]
pub struct World {
    world: WorldHandle,
}

impl World {
    pub fn new(world: WorldHandle) -> Self {
        Self { world }
    }
}

impl TealData for World {
    fn add_methods<T: tealr::mlu::TealDataMethods<Self>>(methods: &mut T) {
        methods.add_method("quake", |_, this, ()| {
            this.world.quake(1);
            Ok(())
        });

        methods.add_method("on_tick", |lua, _, func: Function| {
            // Save callback into globals on events['on_tick']
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
            script_table.set(callback_id, func)?;

            tick_table.set(script_name.to_string(), script_table)?;
            events.set("on_tick", tick_table)?;
            globals.set("events", events)?;

            Ok(())
        });
    }
}
