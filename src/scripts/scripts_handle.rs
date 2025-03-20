use std::{collections::HashSet, path::Path, time::Duration};

use notify::RecursiveMode;
use notify_debouncer_full::new_debouncer;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use crate::world::WorldHandle;

use super::{Command, Scripts};

#[derive(Debug)]
pub struct ScriptsHandle {
    tx: UnboundedSender<Command>,
}

impl ScriptsHandle {
    pub fn new(world: WorldHandle) -> Self {
        let (tx, rx) = unbounded_channel();
        let scripts = match Scripts::new(rx, world) {
            Ok(scripts) => scripts,
            Err(e) => panic!("Failed to load scripts: {}", e),
        };

        tokio::spawn(run_scripts(scripts));

        Self { tx }
    }

    pub fn tick(&self) {
        let _ = self.tx.send(Command::Tick);
    }
}

async fn run_scripts(mut scripts: Scripts) {
    let (tx, mut rx) = unbounded_channel(); // Use tokio's async channel

    // Use a blocking thread to send events into the async channel
    let tx_clone = tx.clone();
    std::thread::spawn(move || {
        // setup debouncer
        let mut debouncer = new_debouncer(
            Duration::from_secs(2),
            None,
            move |res: Result<Vec<notify_debouncer_full::DebouncedEvent>, Vec<notify::Error>>| {
                if let Ok(event) = res {
                    let _ = tx_clone.send(event); // Send event to the async channel
                }
            },
        )
        .expect("Failed to create watcher");

        debouncer
            .watch(Path::new("data/scripts"), RecursiveMode::NonRecursive)
            .expect("Failed to watch scripts directory");

        loop {
            std::thread::sleep(std::time::Duration::from_secs(1)); // Keep the thread alive
        }
    });

    loop {
        tokio::select! {
            Some(command) = scripts.rx.recv() => {
                scripts.handle_command(command).await;
            }
            Some(events) = rx.recv() => {
                let mut reloads = HashSet::new();
                let mut removals = HashSet::new();

                for e in events {
                    for path in &e.event.paths {
                        let extension = match path.extension() {
                            Some(extension) => extension.to_str().unwrap(),
                            None => "",
                        };

                        if extension == "lua" {
                            if e.event.kind.is_create() || e.event.kind.is_modify() {
                                reloads.insert(path.clone());
                            } else if e.event.kind.is_remove() {
                                removals.insert(path.clone());
                            }
                        }
                    }
                }

                for path in reloads {
                    scripts.reload_script(&path);
                }

                for path in removals {
                    scripts.unload_script(&path);
                }
            }
        }
    }
}
