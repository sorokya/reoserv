use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use arc_swap::ArcSwap;
use config::ConfigError;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use once_cell::sync::Lazy;
use tokio::sync::mpsc;

use crate::SETTINGS;

const DEBOUNCE_DURATION: Duration = Duration::from_millis(500);

pub fn spawn_file_watcher<T: Send + Sync + 'static>(
    paths: Vec<PathBuf>,
    arc_swap: &'static Lazy<ArcSwap<T>>,
    reload_fn: fn() -> Result<T, ConfigError>,
    name: &'static str,
) {
    let (tx, mut rx) = mpsc::unbounded_channel::<Event>();

    let mut watcher: RecommendedWatcher =
        RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            notify::Config::default(),
        )
        .expect("Failed to create config file watcher");

    let mut watched_count = 0;
    for path in &paths {
        if path.exists() {
            watcher
                .watch(path, RecursiveMode::NonRecursive)
                .unwrap_or_else(|_| panic!("Failed to watch config file: {:?}", path));
            watched_count += 1;
        }
    }

    if watched_count > 0 {
        tracing::debug!("Watching {} file(s) for config: {}", watched_count, name);
    }

    tokio::spawn(async move {
        let mut last_event: Option<Instant> = None;
        let mut debounce_timer = tokio::time::interval(DEBOUNCE_DURATION);
        debounce_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        debounce_timer.tick().await;

        loop {
            tokio::select! {
                Some(event) = rx.recv() => {
                    let has_modify = event.kind.is_modify() || event.kind.is_create();
                    if has_modify {
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

pub fn spawn_lang_watcher(
    arc_swap: &'static Lazy<ArcSwap<crate::lang::Lang>>,
) {
    let (tx, mut rx) = mpsc::unbounded_channel::<Event>();

    let mut watcher: RecommendedWatcher =
        RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            notify::Config::default(),
        )
        .expect("Failed to create lang file watcher");

    let mut current_lang = SETTINGS.load().server.lang.clone();
    let mut current_path = PathBuf::from(format!("config/lang/{}.ron", current_lang));

    if current_path.exists() {
        watcher
            .watch(&current_path, RecursiveMode::NonRecursive)
            .unwrap_or_else(|_| panic!("Failed to watch lang file: {:?}", current_path));
    }

    tracing::debug!("Watching lang file: {:?}", current_path);

    tokio::spawn(async move {
        let mut last_event: Option<Instant> = None;
        let mut debounce_timer = tokio::time::interval(DEBOUNCE_DURATION);
        debounce_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        debounce_timer.tick().await;

        loop {
            tokio::select! {
                Some(event) = rx.recv() => {
                    let has_modify = event.kind.is_modify() || event.kind.is_create();
                    if has_modify {
                        last_event = Some(Instant::now());
                    }
                }
                _ = debounce_timer.tick() => {
                    let lang = &SETTINGS.load().server.lang;
                    if *lang != current_lang {
                        let _ = watcher.unwatch(&current_path);
                        current_lang = lang.clone();
                        current_path = PathBuf::from(format!("config/lang/{}.ron", current_lang));
                        if current_path.exists() {
                            watcher
                                .watch(&current_path, RecursiveMode::NonRecursive)
                                .unwrap_or_else(|_| panic!("Failed to watch lang file: {:?}", current_path));
                            tracing::debug!("Switched lang watch to: {:?}", current_path);
                        }
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
