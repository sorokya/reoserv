// Avoid musl's default allocator due to lackluster performance
// https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance
#[cfg(target_env = "musl")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[macro_use]
extern crate serde_derive;

use std::{collections::HashMap, time::Duration};

use arc_swap::ArcSwap;
use chrono::Utc;
use eolib::protocol::r#pub::{
    Ecf, Eif, Enf, Esf,
    server::{DropFile, InnFile, ShopFile, SkillMasterFile, TalkFile},
};
use eoplus::Quest;
use once_cell::sync::Lazy;

#[macro_use]
mod utils;
mod arenas;
mod character;
mod db;
mod deep;
use arenas::Arenas;
mod commands;
use commands::Commands;
mod player_commands;
use player::Socket;
use player_commands::PlayerCommands;
mod connection_log;
mod formulas;
use formulas::Formulas;
mod emails;
mod errors;
mod lang;
mod map;
mod player;
mod settings;
use settings::Settings;
mod packet_rate_limits;
use packet_rate_limits::PacketRateLimits;
mod global_drops;
use global_drops::GlobalDrops;
mod sln;
use sln::ping_sln;
mod world;

use tokio::{net::TcpListener, signal, time};
use tokio_tungstenite::accept_async;
use tracing_subscriber::{EnvFilter, fmt::time::ChronoLocal};
use world::WorldHandle;

use crate::{
    db::{Connection, DbHandle},
    emails::Emails,
    lang::Lang,
    player::PlayerHandle,
    utils::{
        load_class_file, load_drop_file, load_inn_file, load_item_file, load_npc_file, load_quests,
        load_shop_file, load_skill_master_file, load_spell_file, load_talk_file,
    },
};

static SETTINGS: Lazy<ArcSwap<Settings>> =
    Lazy::new(|| ArcSwap::from_pointee(Settings::new().expect("Failed to load settings")));

static ARENAS: Lazy<ArcSwap<Arenas>> =
    Lazy::new(|| ArcSwap::from_pointee(Arenas::new().expect("Failed to load arenas")));

static PACKET_RATE_LIMITS: Lazy<ArcSwap<PacketRateLimits>> = Lazy::new(|| {
    ArcSwap::from_pointee(PacketRateLimits::new().expect("Failed to load packet rate limits!"))
});

static COMMANDS: Lazy<ArcSwap<Commands>> =
    Lazy::new(|| ArcSwap::from_pointee(Commands::new().expect("Failed to load commands!")));

static PLAYER_COMMANDS: Lazy<ArcSwap<PlayerCommands>> = Lazy::new(|| {
    ArcSwap::from_pointee(PlayerCommands::new().expect("Failed to load player commands!"))
});

static FORMULAS: Lazy<ArcSwap<Formulas>> =
    Lazy::new(|| ArcSwap::from_pointee(Formulas::new().expect("Failed to load formulas!")));

static LANG: Lazy<ArcSwap<Lang>> =
    Lazy::new(|| ArcSwap::from_pointee(Lang::new().expect("Failed to load lang!")));

static EMAILS: Lazy<ArcSwap<Emails>> =
    Lazy::new(|| ArcSwap::from_pointee(Emails::new().expect("Failed to load emails!")));

static GLOBAL_DROPS: Lazy<ArcSwap<GlobalDrops>> =
    Lazy::new(|| ArcSwap::from_pointee(GlobalDrops::new().expect("Failed to load global drops!")));

static CLASS_DB: Lazy<ArcSwap<Ecf>> =
    Lazy::new(|| ArcSwap::from_pointee(load_class_file().expect("Failed to load ECF file!")));

static DROP_DB: Lazy<ArcSwap<DropFile>> =
    Lazy::new(|| ArcSwap::from_pointee(load_drop_file().expect("Failed to load Drop file!")));

static INN_DB: Lazy<ArcSwap<InnFile>> =
    Lazy::new(|| ArcSwap::from_pointee(load_inn_file().expect("Failed to load Inn file!")));

static ITEM_DB: Lazy<ArcSwap<Eif>> =
    Lazy::new(|| ArcSwap::from_pointee(load_item_file().expect("Failed to load EIF file!")));

static NPC_DB: Lazy<ArcSwap<Enf>> =
    Lazy::new(|| ArcSwap::from_pointee(load_npc_file().expect("Failed to load ENF file!")));

static SHOP_DB: Lazy<ArcSwap<ShopFile>> =
    Lazy::new(|| ArcSwap::from_pointee(load_shop_file().expect("Failed to load Shop file!")));

static SKILL_MASTER_DB: Lazy<ArcSwap<SkillMasterFile>> = Lazy::new(|| {
    ArcSwap::from_pointee(load_skill_master_file().expect("Failed to load Skill Master file!"))
});

static SPELL_DB: Lazy<ArcSwap<Esf>> =
    Lazy::new(|| ArcSwap::from_pointee(load_spell_file().expect("Failed to load ESF file!")));

static TALK_DB: Lazy<ArcSwap<TalkFile>> =
    Lazy::new(|| ArcSwap::from_pointee(load_talk_file().expect("Failed to load Talk file!")));

static QUEST_DB: Lazy<ArcSwap<HashMap<i32, Quest>>> =
    Lazy::new(|| ArcSwap::from_pointee(load_quests()));

static EXP_TABLE: Lazy<ArcSwap<[i32; 254]>> = Lazy::new(|| ArcSwap::from_pointee(load_exp_table()));

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "console")]
    console_subscriber::init();

    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "info");
        }
    }

    tracing_subscriber::fmt()
        .with_timer(ChronoLocal::new(String::from("%Y-%m-%d %I:%M:%S%.3f %p")))
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    println!(
        "__________
\\______   \\ ____  ____  ______ ______________  __
 |       _// __ \\/  _ \\/  ___// __ \\_  __ \\  \\/ /
 |    |   \\  ___(  <_> )___ \\\\  ___/|  | \\/\\   /
 |____|_  /\\___  >____/____  >\\___  >__|    \\_/
        \\/     \\/          \\/     \\/\nThe rusty endless online server: v{}\n",
        include_str!("../VERSION.txt")
    );

    let db = DbHandle::new(match SETTINGS.load().database.driver.as_str() {
        "mysql" => {
            let url = format!(
                "mysql://{}:{}@{}:{}/{}",
                SETTINGS.load().database.username,
                SETTINGS.load().database.password,
                SETTINGS.load().database.host,
                SETTINGS.load().database.port,
                SETTINGS.load().database.name
            );
            let conn = mysql_async::Conn::from_url(&url).await.unwrap();
            Connection::Mysql(crate::db::MysqlConnection { conn, url })
        }
        "sqlite" => Connection::Sqlite(
            rusqlite::Connection::open(format!("{}.db", SETTINGS.load().database.name)).unwrap(),
        ),
        other => panic!("Unsupported database driver: {}", other),
    });

    crate::db::run_startup_migrations(&db, SETTINGS.load().database.driver.as_str()).await?;

    if let Some(row) = db
        .query_one(
            "SELECT (SELECT COUNT(1) FROM accounts),
	   (SELECT COUNT(1) FROM characters),
	   (SELECT COUNT(1) FROM characters WHERE admin_level > 0),
	   (SELECT COUNT(1) FROM guilds);",
        )
        .await?
    {
        tracing::info!("Accounts: {}", row.get_int(0).unwrap_or(0));
        tracing::info!(
            "Characters: {} (Admins: {})",
            row.get_int(1).unwrap_or(0),
            row.get_int(2).unwrap_or(0)
        );
        tracing::info!("Guilds: {}", row.get_int(3).unwrap_or(0));
    }

    let class_db = CLASS_DB.load();
    let item_db = ITEM_DB.load();
    let npc_db = NPC_DB.load();
    let spell_db = SPELL_DB.load();
    let quest_db = QUEST_DB.load();
    tracing::info!("Classes: {}", class_db.classes.len());
    tracing::info!("Items: {}", item_db.items.len());
    tracing::info!("NPCs: {}", npc_db.npcs.len());
    tracing::info!("Skills: {}", spell_db.skills.len());
    tracing::info!("Quests: {}", quest_db.len());

    let world = WorldHandle::new(db.clone());
    {
        let world = world.clone();
        world
            .load_maps()
            .await
            .expect("Failed to load maps. Timeout");
    }

    let mut tick_interval = time::interval(Duration::from_millis(
        SETTINGS.load().world.tick_rate as u64,
    ));
    let tick_world = world.clone();
    tokio::spawn(async move {
        loop {
            tick_interval.tick().await;
            tick_world.tick();
        }
    });

    if SETTINGS.load().server.save_rate > 0 {
        let mut save_interval = time::interval(Duration::from_secs(
            SETTINGS.load().server.save_rate as u64 * 60,
        ));
        let save_world = world.clone();
        tokio::spawn(async move {
            loop {
                save_interval.tick().await;
                save_world.save();
            }
        });
    }

    if SETTINGS.load().sln.enabled {
        let mut sln_interval =
            time::interval(Duration::from_secs(SETTINGS.load().sln.rate as u64 * 60));
        tokio::spawn(async move {
            loop {
                sln_interval.tick().await;
                ping_sln().await;
            }
        });
    }

    let tcp_listener = TcpListener::bind(format!(
        "{}:{}",
        SETTINGS.load().server.host,
        SETTINGS.load().server.port
    ))
    .await
    .unwrap();
    tracing::info!(
        "listening at {}:{}",
        SETTINGS.load().server.host,
        SETTINGS.load().server.port
    );

    let mut websocket_listener = None;
    if !SETTINGS.load().server.websocket_port.is_empty() {
        websocket_listener = Some(
            TcpListener::bind(format!(
                "{}:{}",
                SETTINGS.load().server.host,
                SETTINGS.load().server.websocket_port
            ))
            .await
            .unwrap(),
        );

        tracing::info!(
            "listening for websockets at {}:{}",
            SETTINGS.load().server.host,
            SETTINGS.load().server.websocket_port
        );
    }

    let mut server_world = world.clone();
    let server_db = db.clone();
    tokio::spawn(async move {
        while server_world.is_alive {
            let (socket, addr) = tcp_listener.accept().await.unwrap();
            let ip = addr.ip().to_string();
            let now = Utc::now();

            let player_count = server_world
                .get_connection_count()
                .await
                .expect("Failed to get connection count. Timeout");
            if player_count >= SETTINGS.load().server.max_connections {
                tracing::warn!("{} has been disconnected because the server is full", addr);
                continue;
            }

            // Check reconnect rate limiting
            match server_world.get_ip_last_connect(&ip).await {
                Ok(Some(last_connect)) => {
                    let time_since_last_connect = now - last_connect;
                    if SETTINGS.load().server.ip_reconnect_limit != 0
                        && time_since_last_connect.num_seconds()
                            < SETTINGS.load().server.ip_reconnect_limit.into()
                    {
                        tracing::warn!(
                            "{} has been disconnected because it reconnected too quickly",
                            addr
                        );
                        continue;
                    }
                }
                Ok(None) => {
                    // First connection from this IP, allow it
                }
                Err(e) => {
                    tracing::warn!("Failed to check last connect time for {}: {}", ip, e);
                    // Continue anyway - we have other rate limiting checks
                }
            }

            let num_of_connections = server_world
                .get_ip_connection_count(&ip)
                .await
                .expect("Failed to get IP connection count. Timeout");
            if SETTINGS.load().server.max_connections_per_ip != 0
                && num_of_connections >= SETTINGS.load().server.max_connections_per_ip
            {
                tracing::warn!(
                    "{} has been disconnected because there are already {} connections from {}",
                    addr,
                    num_of_connections,
                    ip
                );
                continue;
            }

            server_world
                .add_connection(&ip)
                .await
                .expect("Failed to add connection. Timeout");

            let player_id = server_world
                .get_next_player_id()
                .await
                .expect("Failed to get next player id. Timeout");

            let player = PlayerHandle::new(
                player_id,
                Socket::Standard(socket),
                ip,
                now,
                server_world.clone(),
                server_db.clone(),
            );
            server_world
                .add_player(player_id, player)
                .await
                .expect("Failed to add player. Timeout");

            tracing::info!(
                "connection accepted ({}) {}/{}",
                addr,
                server_world
                    .get_connection_count()
                    .await
                    .expect("Failed to get connection count. Timeout"),
                SETTINGS.load().server.max_connections
            );
        }
    });

    if let Some(websocket_listener) = websocket_listener {
        let mut websocket_world = world.clone();
        tokio::spawn(async move {
            while websocket_world.is_alive {
                let (socket, addr) = websocket_listener.accept().await.unwrap();
                let websocket = match accept_async(socket).await {
                    Ok(ws) => ws,
                    Err(e) => {
                        tracing::error!("Failed to accept websocket: {}", e);
                        continue;
                    }
                };

                let ip = addr.ip().to_string();
                let now = Utc::now();

                let player_count = websocket_world
                    .get_connection_count()
                    .await
                    .expect("Failed to get connection count. Timeout");
                if player_count >= SETTINGS.load().server.max_connections {
                    tracing::warn!("{} has been disconnected because the server is full", addr);
                    continue;
                }

                // Check reconnect rate limiting
                match websocket_world.get_ip_last_connect(&ip).await {
                    Ok(Some(last_connect)) => {
                        let time_since_last_connect = now - last_connect;
                        if SETTINGS.load().server.ip_reconnect_limit != 0
                            && time_since_last_connect.num_seconds()
                                < SETTINGS.load().server.ip_reconnect_limit.into()
                        {
                            tracing::warn!(
                                "{} has been disconnected because it reconnected too quickly",
                                addr
                            );
                            continue;
                        }
                    }
                    Ok(None) => {
                        // First connection from this IP, allow it
                    }
                    Err(e) => {
                        tracing::warn!("Failed to check last connect time for {}: {}", ip, e);
                        // Continue anyway - we have other rate limiting checks
                    }
                }

                let num_of_connections = websocket_world
                    .get_ip_connection_count(&ip)
                    .await
                    .expect("Failed to get IP connection count. Timeout");
                if SETTINGS.load().server.max_connections_per_ip != 0
                    && num_of_connections >= SETTINGS.load().server.max_connections_per_ip
                {
                    tracing::warn!(
                        "{} has been disconnected because there are already {} connections from {}",
                        addr,
                        num_of_connections,
                        ip
                    );
                    continue;
                }

                websocket_world
                    .add_connection(&ip)
                    .await
                    .expect("Failed to add connection. Timeout");

                let player_id = websocket_world
                    .get_next_player_id()
                    .await
                    .expect("Failed to get next player id. Timeout");

                let player = PlayerHandle::new(
                    player_id,
                    Socket::Web(websocket),
                    ip,
                    now,
                    websocket_world.clone(),
                    db.clone(),
                );
                websocket_world
                    .add_player(player_id, player)
                    .await
                    .expect("Failed to add player. Timeout");

                tracing::info!(
                    "websocket connection accepted ({}) {}/{}",
                    addr,
                    websocket_world
                        .get_connection_count()
                        .await
                        .expect("Failed to get connection count. Timeout"),
                    SETTINGS.load().server.max_connections
                );
            }
        });
    }

    tokio::select! {
        ctrl_c = signal::ctrl_c() => if let Err(err) = ctrl_c {
            tracing::error!("Unable to listen for shutdown signal: {}", err);
        },
        close = close() => if let Err(err) = close {
            tracing::error!("Unable to listen for shutdown signal: {}", err);
        }
    }

    tracing::info!("Shutting down server...");
    world.shutdown().await.expect("Failed to shutdown. Timeout");

    Ok(())
}

#[cfg(windows)]
async fn close() -> anyhow::Result<()> {
    let mut close_stream = signal::windows::ctrl_close()?;
    close_stream.recv().await;
    Ok(())
}

#[cfg(unix)]
async fn close() -> anyhow::Result<()> {
    let mut stream = signal::unix::signal(signal::unix::SignalKind::terminate())?;
    let _ = stream.recv().await;
    Ok(())
}

fn load_exp_table() -> [i32; 254] {
    let mut exp_table = [0; 254];

    for (i, exp) in exp_table.iter_mut().enumerate() {
        *exp = ((i as f64).powf(3.0) * 133.1).round() as i32;
    }

    exp_table
}
