use mlua::{Function, Table};

use super::super::Scripts;

impl Scripts {
    pub fn tick(&mut self) {
        let globals = self.lua.globals();

        let events: Table = globals
            .get("events")
            .unwrap_or_else(|_| self.lua.create_table().unwrap());

        let tick_table: Table = events
            .get("on_player_command")
            .unwrap_or_else(|_| self.lua.create_table().unwrap());

        for (_, table) in tick_table.pairs::<String, Table>().flatten() {
            for (_, func) in table.pairs::<i32, Function>().flatten() {
                if let Err(err) = func.call::<()>(()) {
                    error!("Error in on_tick callback: {}", err);
                }
            }
        }
    }
}
