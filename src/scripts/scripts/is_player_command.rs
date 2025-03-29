use mlua::{Function, Table};

use super::super::Scripts;

impl Scripts {
    pub fn handle_player_command(&mut self, args: &[String]) -> bool {
        if args.is_empty() {
            return false;
        }

        let name = match args.get(0) {
            Some(name) => name,
            None => return false,
        };

        let globals = self.lua.globals();

        let events: Table = globals
            .get("events")
            .unwrap_or_else(|_| self.lua.create_table().unwrap());

        let command_table: Table = events
            .get("on_player_command")
            .unwrap_or_else(|_| self.lua.create_table().unwrap());

        command_table.pairs::<String, Table>().any(|pair| {
            pair.ok()
                .and_then(|(_, table)| {
                    table
                        .pairs::<String, Function>()
                        .find_map(|pair| pair.ok().map(|(cmd, _)| cmd == *name))
                })
                .unwrap_or(false)
        })
    }
}
