const VERSION: &str = "1.10.4";

// Avoid musl's default allocator due to lackluster performance
// https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance
#[cfg(target_env = "musl")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use std::{collections::HashMap, time::Duration};

use chrono::Utc;
use eolib::protocol::r#pub::{
    server::{DropFile, InnFile, ShopFile, SkillMasterFile, TalkFile},
    Ecf, Eif, Enf, Esf,
};
use eoplus::Quest;
use lazy_static::lazy_static;

#[macro_use]
mod utils;
mod arenas;
mod character;
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
mod sln;
use sln::ping_sln;
mod world;
use mysql_async::prelude::*;

use tokio::{net::TcpListener, signal, time};
use tokio_tungstenite::accept_async;
use world::WorldHandle;

use crate::{
    emails::Emails,
    lang::Lang,
    player::PlayerHandle,
    utils::{
        load_class_file, load_drop_file, load_inn_file, load_item_file, load_npc_file, load_quests,
        load_shop_file, load_skill_master_file, load_spell_file, load_talk_file,
    },
};

lazy_static! {
    static ref SETTINGS: Settings = Settings::new().expect("Failed to load settings!");
    static ref ARENAS: Arenas = Arenas::new().expect("Failed to load arenas!");
    static ref PACKET_RATE_LIMITS: PacketRateLimits =
        PacketRateLimits::new().expect("Failed to load packet rate limits!");
    static ref COMMANDS: Commands = Commands::new().expect("Failed to load commands!");
    static ref PLAYER_COMMANDS: PlayerCommands =
        PlayerCommands::new().expect("Failed to load player commands!");
    static ref FORMULAS: Formulas = Formulas::new().expect("Failed to load formulas!");
    static ref LANG: Lang = Lang::new().expect("Failed to load lang!");
    static ref EMAILS: Emails = Emails::new().expect("Failed to load emails!");
    static ref CLASS_DB: Ecf = load_class_file().expect("Failed to load ECF file!");
    static ref DROP_DB: DropFile = load_drop_file().expect("Failed to load Drop file!");
    static ref INN_DB: InnFile = load_inn_file().expect("Failed to load Inn file!");
    static ref ITEM_DB: Eif = load_item_file().expect("Failed to load EIF file!");
    static ref NPC_DB: Enf = load_npc_file().expect("Failed to load ENF file!");
    static ref SHOP_DB: ShopFile = load_shop_file().expect("Failed to load Shop file!");
    static ref SKILL_MASTER_DB: SkillMasterFile =
        load_skill_master_file().expect("Failed to load Skill Master file!");
    static ref SPELL_DB: Esf = load_spell_file().expect("Failed to load ESF file!");
    static ref TALK_DB: TalkFile = load_talk_file().expect("Failed to load Talk file!");
    static ref QUEST_DB: HashMap<i32, Quest> = load_quests();
    static ref EXP_TABLE: [i32; 254] = load_exp_table();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "console")]
    console_subscriber::init();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();
    println!(
        "__________
\\______   \\ ____  ____  ______ ______________  __
 |       _// __ \\/  _ \\/  ___// __ \\_  __ \\  \\/ /
 |    |   \\  ___(  <_> )___ \\\\  ___/|  | \\/\\   /
 |____|_  /\\___  >____/____  >\\___  >__|    \\_/
        \\/     \\/          \\/     \\/\nThe rusty endless online server: v{}\n",
        VERSION
    );

    let database_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        SETTINGS.database.username,
        SETTINGS.database.password,
        SETTINGS.database.host,
        SETTINGS.database.port,
        SETTINGS.database.name
    );

    let pool = mysql_async::Pool::new(mysql_async::Opts::from_url(&database_url).unwrap());
    {
        let conn = pool
            .get_conn()
            .await
            .expect("Failed to get connection from pool! Is MySQL running?");
        let mut results = r"SELECT
        (SELECT COUNT(*) FROM `Account`) 'accounts',
        (SELECT COUNT(*) FROM `Character`) 'characters',
        (SELECT COUNT(*) FROM `Guild`) 'guilds',
        (SELECT COUNT(*) FROM `Character` WHERE `admin_level` > 0) 'admins'"
            .with(())
            .run(conn)
            .await?;

        results
            .for_each(|row| {
                info!("Accounts: {}", row.get::<i64, usize>(0).unwrap());
                info!(
                    "Characters: {} (Admins: {})",
                    row.get::<i64, usize>(1).unwrap(),
                    row.get::<i64, usize>(3).unwrap()
                );
                info!("Guilds: {}", row.get::<i64, usize>(2).unwrap());
            })
            .await
            .unwrap();
    }

    info!("Classes: {}", CLASS_DB.classes.len());
    info!("Items: {}", ITEM_DB.items.len());
    info!("NPCs: {}", NPC_DB.npcs.len());
    info!("Skills: {}", SPELL_DB.skills.len());
    info!("Quests: {}", QUEST_DB.len());

    let world = WorldHandle::new(pool.clone());
    {
        let world = world.clone();
        world.load_maps().await;
    }

    let mut tick_interval = time::interval(Duration::from_millis(SETTINGS.world.tick_rate as u64));
    let tick_world = world.clone();
    tokio::spawn(async move {
        loop {
            tick_interval.tick().await;
            tick_world.tick();
        }
    });

    if SETTINGS.server.save_rate > 0 {
        let mut save_interval =
            time::interval(Duration::from_secs(SETTINGS.server.save_rate as u64 * 60));
        let save_world = world.clone();
        tokio::spawn(async move {
            loop {
                save_interval.tick().await;
                save_world.save();
            }
        });
    }

    if SETTINGS.sln.enabled {
        let mut sln_interval = time::interval(Duration::from_secs(SETTINGS.sln.rate as u64 * 60));
        tokio::spawn(async move {
            loop {
                sln_interval.tick().await;
                ping_sln().await;
            }
        });
    }

    let tcp_listener =
        TcpListener::bind(format!("{}:{}", SETTINGS.server.host, SETTINGS.server.port))
            .await
            .unwrap();
    info!(
        "listening at {}:{}",
        SETTINGS.server.host, SETTINGS.server.port
    );

    let mut websocket_listener = None;
    if !SETTINGS.server.websocket_port.is_empty() {
        websocket_listener = Some(
            TcpListener::bind(format!(
                "{}:{}",
                SETTINGS.server.host, SETTINGS.server.websocket_port
            ))
            .await
            .unwrap(),
        );

        info!(
            "listening for websockets at {}:{}",
            SETTINGS.server.host, SETTINGS.server.websocket_port
        );
    }

    let mut server_world = world.clone();
    let server_pool = pool.clone();
    tokio::spawn(async move {
        while server_world.is_alive {
            let (socket, addr) = tcp_listener.accept().await.unwrap();
            let ip = addr.ip().to_string();
            let now = Utc::now();

            let player_count = server_world.get_connection_count().await;
            if player_count >= SETTINGS.server.max_connections {
                warn!("{} has been disconnected because the server is full", addr);
                continue;
            }

            if let Some(last_connect) = server_world.get_ip_last_connect(&ip).await {
                let time_since_last_connect = now - last_connect;
                if SETTINGS.server.ip_reconnect_limit != 0
                    && time_since_last_connect.num_seconds()
                        < SETTINGS.server.ip_reconnect_limit.into()
                {
                    warn!(
                        "{} has been disconnected because it reconnected too quickly",
                        addr
                    );
                    continue;
                }
            }

            let num_of_connections = server_world.get_ip_connection_count(&ip).await;
            if SETTINGS.server.max_connections_per_ip != 0
                && num_of_connections >= SETTINGS.server.max_connections_per_ip
            {
                warn!(
                    "{} has been disconnected because there are already {} connections from {}",
                    addr, num_of_connections, ip
                );
                continue;
            }

            server_world.add_connection(&ip).await;

            let player_id = server_world.get_next_player_id().await.unwrap();

            let player = PlayerHandle::new(
                player_id,
                Socket::Standard(socket),
                ip,
                now,
                server_world.clone(),
                server_pool.clone(),
            );
            server_world.add_player(player_id, player).await.unwrap();

            info!(
                "connection accepted ({}) {}/{}",
                addr,
                server_world.get_connection_count().await,
                SETTINGS.server.max_connections
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
                        error!("Failed to accept websocket: {}", e);
                        continue;
                    }
                };

                let ip = addr.ip().to_string();
                let now = Utc::now();

                let player_count = websocket_world.get_connection_count().await;
                if player_count >= SETTINGS.server.max_connections {
                    warn!("{} has been disconnected because the server is full", addr);
                    continue;
                }

                if let Some(last_connect) = websocket_world.get_ip_last_connect(&ip).await {
                    let time_since_last_connect = now - last_connect;
                    if SETTINGS.server.ip_reconnect_limit != 0
                        && time_since_last_connect.num_seconds()
                            < SETTINGS.server.ip_reconnect_limit.into()
                    {
                        warn!(
                            "{} has been disconnected because it reconnected too quickly",
                            addr
                        );
                        continue;
                    }
                }

                let num_of_connections = websocket_world.get_ip_connection_count(&ip).await;
                if SETTINGS.server.max_connections_per_ip != 0
                    && num_of_connections >= SETTINGS.server.max_connections_per_ip
                {
                    warn!(
                        "{} has been disconnected because there are already {} connections from {}",
                        addr, num_of_connections, ip
                    );
                    continue;
                }

                websocket_world.add_connection(&ip).await;

                let player_id = websocket_world.get_next_player_id().await.unwrap();

                let player = PlayerHandle::new(
                    player_id,
                    Socket::Web(websocket),
                    ip,
                    now,
                    websocket_world.clone(),
                    pool.clone(),
                );
                websocket_world.add_player(player_id, player).await.unwrap();

                info!(
                    "websocket connection accepted ({}) {}/{}",
                    addr,
                    websocket_world.get_connection_count().await,
                    SETTINGS.server.max_connections
                );
            }
        });
    }

    tokio::select! {
        ctrl_c = signal::ctrl_c() => match ctrl_c {
            Ok(()) => {},
            Err(err) => {
                error!("Unable to listen for shutdown signal: {}", err);
            }
        },
        close = close() => match close {
            Ok(()) => {},
            Err(err) => {
                error!("Unable to listen for shutdown signal: {}", err);
            }
        }
    }

    info!("Shutting down server...");
    world.shutdown().await;

    Ok(())
}

#[cfg(windows)]
async fn close() -> Result<(), Box<dyn std::error::Error>> {
    let mut close_stream = signal::windows::ctrl_close()?;
    close_stream.recv().await;
    Ok(())
}

#[cfg(unix)]
async fn close() -> Result<(), Box<dyn std::error::Error>> {
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
