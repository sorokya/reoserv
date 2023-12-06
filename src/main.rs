const VERSION: &str = "0.0.0";

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use std::time::Duration;

use lazy_static::lazy_static;

mod arenas;
mod character;
use arenas::Arenas;
mod commands;
use commands::Commands;
mod formulas;
use formulas::Formulas;
mod errors;
mod handlers;
mod lang;
mod map;
mod player;
mod settings;
use settings::Settings;
mod sln;
use sln::ping_sln;
#[macro_use]
mod utils;
mod world;

use eo::{
    data::EOInt,
    pubs::{
        DropFile, EcfFile, EifFile, EnfFile, EsfFile, InnFile, ShopFile, SkillMasterFile, TalkFile,
    },
};
use mysql_async::prelude::*;

use tokio::{net::TcpListener, signal, time};
use world::WorldHandle;

use crate::{
    lang::Lang,
    player::PlayerHandle,
    utils::{
        load_class_file, load_drop_file, load_inn_file, load_item_file, load_npc_file,
        load_shop_file, load_skill_master_file, load_spell_file, load_talk_file,
    },
};

lazy_static! {
    static ref SETTINGS: Settings = Settings::new().expect("Failed to load settings!");
    static ref ARENAS: Arenas = Arenas::new().expect("Failed to load arenas!");
    static ref COMMANDS: Commands = Commands::new().expect("Failed to load commands!");
    static ref FORMULAS: Formulas = Formulas::new().expect("Failed to load formulas!");
    static ref LANG: Lang = Lang::new().expect("Failed to load lang!");
    static ref CLASS_DB: EcfFile = load_class_file().expect("Failed to load ECF file!");
    static ref DROP_DB: DropFile = load_drop_file().expect("Failed to load Drop file!");
    static ref INN_DB: InnFile = load_inn_file().expect("Failed to load Inn file!");
    static ref ITEM_DB: EifFile = load_item_file().expect("Failed to load EIF file!");
    static ref NPC_DB: EnfFile = load_npc_file().expect("Failed to load ENF file!");
    static ref SHOP_DB: ShopFile = load_shop_file().expect("Failed to load Shop file!");
    static ref SKILL_MASTER_DB: SkillMasterFile =
        load_skill_master_file().expect("Failed to load Skill Master file!");
    static ref SPELL_DB: EsfFile = load_spell_file().expect("Failed to load ESF file!");
    static ref TALK_DB: TalkFile = load_talk_file().expect("Failed to load Talk file!");
    static ref EXP_TABLE: [EOInt; 254] = load_exp_table();
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

    info!("Classes: {}", CLASS_DB.num_classes);
    info!("Items: {}", ITEM_DB.num_items);
    info!("NPCs: {}", NPC_DB.num_npcs);
    info!("Spells: {}", SPELL_DB.num_spells);

    let world = WorldHandle::new(pool.clone());
    {
        let world = world.clone();
        world.load_maps().await;
    }

    let mut ping_interval = time::interval(Duration::from_secs(SETTINGS.server.ping_rate.into()));
    let ping_timer_world = world.clone();
    tokio::spawn(async move {
        loop {
            ping_interval.tick().await;
            ping_timer_world.ping_players();
        }
    });

    let mut tick_interval = time::interval(Duration::from_millis(SETTINGS.world.tick_rate.into()));
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

    let mut server_world = world.clone();
    tokio::spawn(async move {
        while server_world.is_alive {
            let (socket, addr) = tcp_listener.accept().await.unwrap();

            let player_count = server_world.get_player_count().await.unwrap();
            if player_count >= SETTINGS.server.max_connections as usize {
                warn!("{} has been disconnected because the server is full", addr);
                continue;
            }

            let player_id = server_world.get_next_player_id().await.unwrap();

            let player = PlayerHandle::new(player_id, socket, server_world.clone(), pool.clone());
            server_world.add_player(player_id, player).await.unwrap();

            info!(
                "connection accepted ({}) {}/{}",
                addr,
                player_count + 1,
                SETTINGS.server.max_connections
            );
        }
    });

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }

    info!("Shutting down server...");
    world.shutdown().await;

    Ok(())
}

fn load_exp_table() -> [EOInt; 254] {
    let mut exp_table = [0; 254];

    for (i, exp) in exp_table.iter_mut().enumerate() {
        *exp = ((i as f64).powf(3.0) * 133.1).round() as EOInt;
    }

    exp_table
}
