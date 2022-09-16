const VERSION: &str = "0.0.0";

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use std::time::Duration;

use lazy_static::lazy_static;

mod character;
mod commands;
use commands::Commands;
mod formulas;
use formulas::Formulas;
mod errors;
mod map;
mod player;
mod settings;
use settings::Settings;
mod utils;
mod world;

use eo::data::EOByte;
use mysql_async::prelude::*;

use tokio::{net::TcpListener, time};
use world::WorldHandle;

use crate::player::PlayerHandle;

pub type PacketBuf = Vec<EOByte>;

lazy_static! {
    static ref SETTINGS: Settings = Settings::new().expect("Failed to load settings!");
    static ref COMMANDS: Commands = Commands::new().expect("Failed to load commands!");
    static ref FORMULAS: Formulas = Formulas::new().expect("Failed to load formulas!");
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

    let mut world = WorldHandle::new(pool.clone());
    {
        let world = world.clone();
        let _ = tokio::join!(world.load_pubs(), world.load_maps());
    }

    let mut ping_interval = time::interval(Duration::from_secs(SETTINGS.server.ping_rate.into()));
    ping_interval.tick().await;
    let ping_timer_world = world.clone();
    tokio::spawn(async move {
        loop {
            ping_interval.tick().await;
            ping_timer_world.ping_players();
        }
    });

    let mut npc_spawn_interval = time::interval(Duration::from_secs(SETTINGS.npcs.respawn_rate.into()));
    npc_spawn_interval.tick().await;
    let npc_spawn_world = world.clone();
    tokio::spawn(async move {
        loop {
            npc_spawn_interval.tick().await;
            npc_spawn_world.spawn_npcs();
        }
    });

    let mut npc_act_interval = time::interval(Duration::from_millis(SETTINGS.npcs.act_rate.into()));
    npc_act_interval.tick().await;
    let npc_act_world = world.clone();
    tokio::spawn(async move {
        loop {
            npc_act_interval.tick().await;
            npc_act_world.act_npcs();
        }
    });

    let tcp_listener =
        TcpListener::bind(format!("{}:{}", SETTINGS.server.host, SETTINGS.server.port))
            .await
            .unwrap();
    info!(
        "listening at {}:{}",
        SETTINGS.server.host, SETTINGS.server.port
    );

    while world.is_alive {
        let (socket, addr) = tcp_listener.accept().await.unwrap();

        let player_count = world.get_player_count().await.unwrap();
        if player_count >= SETTINGS.server.max_connections as usize {
            warn!("{} has been disconnected because the server is full", addr);
            continue;
        }

        let player_id = world.get_next_player_id().await.unwrap();

        let player = PlayerHandle::new(player_id, socket, world.clone(), pool.clone());
        world.add_player(player_id, player).await.unwrap();

        info!(
            "connection accepted ({}) {}/{}",
            addr,
            player_count + 1,
            SETTINGS.server.max_connections
        );
    }

    Ok(())
}
