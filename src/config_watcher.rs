use std::{
    collections::HashSet,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use arc_swap::ArcSwap;
use config::ConfigError;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use once_cell::sync::Lazy;
use tokio::sync::mpsc;

use crate::{SETTINGS, world::WorldHandle};

const DEBOUNCE_DURATION: Duration = Duration::from_millis(500);

pub fn spawn_dir_watcher<T: Send + Sync + 'static>(
    dir: PathBuf,
    arc_swap: &'static Lazy<ArcSwap<T>>,
    reload_fn: fn() -> Result<T, ConfigError>,
    name: &'static str,
) {
    let (tx, mut rx) = mpsc::unbounded_channel::<Event>();

    let mut watcher: RecommendedWatcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        notify::Config::default(),
    )
    .expect("Failed to create config dir watcher");

    if dir.exists() {
        watcher
            .watch(&dir, RecursiveMode::NonRecursive)
            .unwrap_or_else(|_| panic!("Failed to watch config dir: {:?}", dir));
        tracing::debug!("Watching config dir: {:?}", dir);
    } else {
        tracing::warn!("Config dir not found: {:?}", dir);
    }

    tokio::spawn(async move {
        let _watcher = watcher;
        let mut last_event: Option<Instant> = None;
        let mut debounce_timer = tokio::time::interval(DEBOUNCE_DURATION);
        debounce_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        debounce_timer.tick().await;

        loop {
            tokio::select! {
                Some(event) = rx.recv() => {
                    let relevant = event.paths.iter().any(|p| {
                        p.extension().is_some_and(|e| e == "toml" || e == "ron")
                    });
                    let has_modify = event.kind.is_modify() || event.kind.is_create();
                    if relevant && has_modify {
                        last_event = Some(Instant::now());
                    }
                }
                _ = debounce_timer.tick() => {
                    if let Some(last) = last_event {
                        if last.elapsed() >= DEBOUNCE_DURATION {
                            last_event = None;
                            match reload_fn() {
                                Ok(new_val) => {
                                    arc_swap.store(Arc::new(new_val));
                                    tracing::debug!("Reloaded config: {}", name);
                                }
                                Err(e) => {
                                    tracing::error!("Failed to reload {}: {}", name, e);
                                }
                            }
                        }
                    }
                }
            }
        }
    });
}

pub fn spawn_lang_watcher(arc_swap: &'static Lazy<ArcSwap<crate::lang::Lang>>) {
    let (tx, mut rx) = mpsc::unbounded_channel::<Event>();

    let mut watcher: RecommendedWatcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        notify::Config::default(),
    )
    .expect("Failed to create lang dir watcher");

    let lang_dir = PathBuf::from("config/lang");
    if lang_dir.exists() {
        watcher
            .watch(&lang_dir, RecursiveMode::NonRecursive)
            .unwrap_or_else(|_| panic!("Failed to watch lang dir: {:?}", lang_dir));
        tracing::debug!("Watching lang dir: {:?}", lang_dir);
    }

    let mut current_lang = SETTINGS.load().server.lang.clone();

    tokio::spawn(async move {
        let _watcher = watcher;
        let mut last_event: Option<Instant> = None;
        let mut debounce_timer = tokio::time::interval(DEBOUNCE_DURATION);
        debounce_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        debounce_timer.tick().await;

        loop {
            tokio::select! {
                Some(event) = rx.recv() => {
                    let relevant = event.paths.iter().any(|p| {
                        p.extension().is_some_and(|e| e == "ron")
                    });
                    let has_modify = event.kind.is_modify() || event.kind.is_create();
                    if relevant && has_modify {
                        last_event = Some(Instant::now());
                    }
                }
                _ = debounce_timer.tick() => {
                    let lang = &SETTINGS.load().server.lang;
                    if *lang != current_lang {
                        current_lang = lang.clone();
                        tracing::debug!("Lang switched to: {}", current_lang);
                    }

                    if let Some(last) = last_event {
                        if last.elapsed() >= DEBOUNCE_DURATION {
                            last_event = None;
                            match crate::lang::Lang::reload() {
                                Ok(new_val) => {
                                    arc_swap.store(Arc::new(new_val));
                                    tracing::debug!("Reloaded config: Lang");
                                }
                                Err(e) => {
                                    tracing::error!("Failed to reload Lang: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        }
    });
}

pub fn spawn_map_watcher(world: WorldHandle) {
    let (tx, mut rx) = mpsc::unbounded_channel::<Event>();

    let dir = PathBuf::from("data/maps");
    let mut watcher: RecommendedWatcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        notify::Config::default(),
    )
    .expect("Failed to create map dir watcher");

    if dir.exists() {
        watcher
            .watch(&dir, RecursiveMode::NonRecursive)
            .unwrap_or_else(|_| panic!("Failed to watch maps dir: {:?}", dir));
    } else {
        tracing::warn!("Maps dir not found: {:?}", dir);
    }

    tokio::spawn(async move {
        let _watcher = watcher;
        let mut pending_maps: HashSet<i32> = HashSet::new();
        let mut last_event: Option<Instant> = None;
        let mut debounce_timer = tokio::time::interval(DEBOUNCE_DURATION);
        debounce_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        debounce_timer.tick().await;

        loop {
            tokio::select! {
                Some(event) = rx.recv() => {
                    for path in &event.paths {
                        if path.extension().is_some_and(|e| e == "emf") {
                            if let Some(map_id) = extract_map_id(path) {
                                pending_maps.insert(map_id);
                            }
                        }
                    }
                    last_event = Some(Instant::now());
                }
                _ = debounce_timer.tick() => {
                    if let Some(last) = last_event {
                        if last.elapsed() >= DEBOUNCE_DURATION {
                            last_event = None;
                            let maps_to_reload = std::mem::take(&mut pending_maps);
                            for map_id in maps_to_reload {
                                world.reload_map(map_id);
                                tracing::info!("Reloaded map: {}", map_id);
                            }
                        }
                    }
                }
            }
        }
    });
}

fn extract_map_id(path: &PathBuf) -> Option<i32> {
    let stem = path.file_stem()?.to_str()?;
    let numeric: String = stem.chars().take_while(|c| c.is_ascii_digit()).collect();
    if numeric.is_empty() {
        return None;
    }
    numeric.parse::<i32>().ok()
}
