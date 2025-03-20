use std::{collections::HashSet, fs, path::PathBuf};

use crate::world::WorldHandle;

use super::{Command, World};
use glob::glob;
use tealr::mlu::mlua::{Function, Lua, Table};
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Debug)]
pub struct Scripts {
    pub rx: UnboundedReceiver<Command>,
    lua: Lua,
    loaded_scripts: HashSet<String>,
}

impl Scripts {
    pub fn new(rx: UnboundedReceiver<Command>, world: WorldHandle) -> anyhow::Result<Self> {
        let lua = Lua::new();

        let globals = lua.globals();
        let world = World::new(world);

        globals.set("world", world)?;
        globals.set("events", lua.create_table()?)?;

        let exists = fs::exists("data/scripts")?;
        if !exists {
            fs::create_dir("data/scripts")?;
        }

        let entries = glob("data/scripts/*.lua")?;
        let mut loaded_scripts = HashSet::new();
        for path in entries {
            let path = match path {
                Ok(ref path) => path,
                Err(e) => {
                    error!("Failed to read path: {}", e);
                    continue;
                }
            };

            let script_name = path.file_name().unwrap().to_string_lossy().to_string();
            let script_content = format!(
                r#"
                _SCRIPT_NAME = "{}"
                {}
                "#,
                script_name,
                fs::read_to_string(&path)?
            );

            lua.load(&script_content).exec()?;
            info!("Loaded script: {}", script_name);

            loaded_scripts.insert(script_name);
        }

        Ok(Self {
            rx,
            lua,
            loaded_scripts,
        })
    }

    pub fn unload_script(&mut self, path: &PathBuf) {
        if path.extension().unwrap() != "lua" {
            return;
        }

        let script_name = path.file_name().unwrap().to_string_lossy().to_string();

        if self.loaded_scripts.contains(&script_name) {
            info!("Unloading script: {}", script_name);
            self.clear_script_callbacks(&script_name);
            self.loaded_scripts.remove(&script_name);
        }
    }

    pub fn reload_script(&mut self, path: &PathBuf) {
        let lua = &self.lua;
        if path.extension().unwrap() != "lua" {
            return;
        }

        let script_name = path.file_name().unwrap().to_string_lossy().to_string();

        let existing = self.loaded_scripts.contains(&script_name);

        if existing {
            self.clear_script_callbacks(&script_name);
            self.loaded_scripts.remove(&script_name);
        }

        match std::fs::read_to_string(path) {
            Ok(script_content) => {
                if existing {
                    info!("Reloading script: {}", script_name);
                } else {
                    info!("Loading script: {}", script_name);
                }

                if let Err(err) = lua
                    .load(&format!(
                        r#"
                _SCRIPT_NAME = "{}"
                {}
                "#,
                        script_name, script_content,
                    ))
                    .exec()
                {
                    error!("Error reloading script {}: {}", script_name, err);
                } else {
                    self.loaded_scripts.insert(script_name);
                }
            }
            Err(_) => {
                info!("Unloading script: {}", script_name);
            }
        }
    }

    fn clear_script_callbacks(&self, script_name: &str) {
        let lua = &self.lua;
        let globals = lua.globals();

        let events: Table = globals
            .get("events")
            .unwrap_or_else(|_| lua.create_table().unwrap());

        let mut empty_events = Vec::new();

        for pair in events.pairs::<String, Table>() {
            if let Ok((event_name, event_table)) = pair {
                if event_table.contains_key(script_name).unwrap_or(false) {
                    event_table.raw_remove(script_name).unwrap();
                }

                if event_table.is_empty() {
                    empty_events.push(event_name);
                }
            }
        }

        for event_name in empty_events {
            events.raw_remove(event_name).unwrap();
        }

        globals.set("events", events).unwrap();
    }

    pub async fn handle_command(&mut self, command: Command) {
        match command {
            Command::Tick => {
                let globals = self.lua.globals();

                if let Ok(events) = globals.get::<Table>("events") {
                    if let Ok(tick_table) = events.get::<Table>("on_tick") {
                        for pair in tick_table.pairs::<String, Table>() {
                            if let Ok((_, table)) = pair {
                                for pair in table.pairs::<i32, Function>() {
                                    if let Ok((_, func)) = pair {
                                        if let Err(err) = func.call::<()>(()) {
                                            error!("Error in on_tick callback: {}", err);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
