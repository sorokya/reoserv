const VERSION: &str = "0.0.0";

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use std::{time::Duration, fs::File, io::Read};

use bytes::Bytes;
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
mod sln;
use sln::ping_sln;
mod utils;
mod world;

use eo::{data::{StreamReader, Serializeable, EOInt}, pubs::{EifFile, EnfFile, DropFile, TalkFile, ShopFile, SkillMasterFile, EcfFile, InnFile, EsfFile}};
use mysql_async::prelude::*;

use tokio::{net::TcpListener, time, signal};
use world::WorldHandle;

use crate::player::PlayerHandle;

lazy_static! {
    static ref SETTINGS: Settings = Settings::new().expect("Failed to load settings!");
    static ref COMMANDS: Commands = Commands::new().expect("Failed to load commands!");
    static ref FORMULAS: Formulas = Formulas::new().expect("Failed to load formulas!");
    static ref CLASS_DB: EcfFile = load_class_file().expect("Failed to load ECF file!");
    static ref DROP_DB: DropFile = load_drop_file().expect("Failed to load Drop file!");
    static ref INN_DB: InnFile = load_inn_file().expect("Failed to load Inn file!");
    static ref ITEM_DB: EifFile = load_item_file().expect("Failed to load EIF file!");
    static ref NPC_DB: EnfFile = load_npc_file().expect("Failed to load ENF file!");
    static ref SHOP_DB: ShopFile = load_shop_file().expect("Failed to load Shop file!");
    static ref SKILL_MASTER_DB: SkillMasterFile = load_skill_master_file().expect("Failed to load Skill Master file!");
    static ref SPELL_DB: EsfFile = load_spell_file().expect("Failed to load ESF file!");
    static ref TALK_DB: TalkFile = load_talk_file().expect("Failed to load Talk file!");
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
    info!("Drops: {}", DROP_DB.npcs.iter().map(|npc| npc.num_of_drops as EOInt).sum::<EOInt>());
    info!("Inns: {}", INN_DB.inns.len());
    info!("Items: {}", ITEM_DB.num_items);
    info!("NPCs: {}", NPC_DB.num_npcs);
    info!("Shops: {}", SHOP_DB.shops.len());
    info!("Skill Masters: {}", SKILL_MASTER_DB.skill_masters.len());
    info!("Spells: {}", SPELL_DB.num_spells);
    info!("Noisy NPCs: {}", TALK_DB.npcs.len());

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

    let mut npc_spawn_interval =
        time::interval(Duration::from_secs(SETTINGS.npcs.respawn_rate.into()));
    npc_spawn_interval.tick().await;
    let npc_spawn_world = world.clone();
    tokio::spawn(async move {
        loop {
            npc_spawn_interval.tick().await;
            npc_spawn_world.spawn_npcs();
        }
    });

    let mut npc_act_interval = time::interval(Duration::from_millis(SETTINGS.npcs.act_rate.into()));
    let npc_act_world = world.clone();
    tokio::spawn(async move {
        loop {
            npc_act_interval.tick().await;
            npc_act_world.act_npcs();
        }
    });

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
        Ok(()) => {},
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        },
    }

    info!("Shutting down server...");
    world.shutdown().await;

    Ok(())
}

fn load_class_file() -> Result<EcfFile, Box<dyn std::error::Error>> {
    let mut file = File::open("pub/dat001.ecf")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let bytes = Bytes::from(buf);

    let reader = StreamReader::new(bytes);

    let mut ecf_file = EcfFile::default();
    ecf_file.deserialize(&reader);
    Ok(ecf_file)
}

fn load_drop_file() -> Result<DropFile, Box<dyn std::error::Error>> {
    let mut file = File::open("pub/dtd001.edf")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let bytes = Bytes::from(buf);
    let reader = StreamReader::new(bytes);

    let mut drop_file = DropFile::default();
    drop_file.deserialize(&reader);
    Ok(drop_file)
}

fn load_inn_file() -> Result<InnFile, Box<dyn std::error::Error>> {
    let mut file = File::open("pub/din001.eid")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    
    let bytes = Bytes::from(buf);
    let reader = StreamReader::new(bytes);

    let mut inn_file = InnFile::default();
    inn_file.deserialize(&reader);
    Ok(inn_file)
}

fn load_item_file() -> Result<EifFile, Box<dyn std::error::Error>> {
    let mut file = File::open("pub/dat001.eif")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let bytes = Bytes::from(buf);
    let reader = StreamReader::new(bytes);

    let mut item_file = EifFile::default();
    item_file.deserialize(&reader);
    Ok(item_file)
}

fn load_npc_file() -> Result<EnfFile, Box<dyn std::error::Error>> {
    let mut file = File::open("pub/dtn001.enf")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let bytes = Bytes::from(buf);
    let reader = StreamReader::new(bytes);

    let mut npc_file = EnfFile::default();
    npc_file.deserialize(&reader);
    Ok(npc_file)
}

fn load_shop_file() -> Result<ShopFile, Box<dyn std::error::Error>> {
    let mut file = File::open("pub/dts001.esf")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let bytes = Bytes::from(buf);
    let reader = StreamReader::new(bytes);

    let mut shop_file = ShopFile::default();
    shop_file.deserialize(&reader);
    Ok(shop_file)
}

fn load_skill_master_file() -> Result<SkillMasterFile, Box<dyn std::error::Error>> {
    let mut file = File::open("pub/dsm001.emf")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let bytes = Bytes::from(buf);
    let reader = StreamReader::new(bytes);

    let mut skill_master_file = SkillMasterFile::default();
    skill_master_file.deserialize(&reader);
    Ok(skill_master_file)
}

fn load_spell_file() -> Result<EsfFile, Box<dyn std::error::Error>> {
    let mut file = File::open("pub/dsl001.esf")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let bytes = Bytes::from(buf);
    let reader = StreamReader::new(bytes);

    let mut spell_file = EsfFile::default();
    spell_file.deserialize(&reader);
    Ok(spell_file)
}

fn load_talk_file() -> Result<TalkFile, Box<dyn std::error::Error>> {
    let mut file = File::open("pub/ttd001.etf")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let bytes = Bytes::from(buf);
    let reader = StreamReader::new(bytes);

    let mut talk_file = TalkFile::default();
    talk_file.deserialize(&reader);
    Ok(talk_file)
}