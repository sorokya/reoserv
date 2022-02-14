const VERSION: &str = "0.0.0";

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use lazy_static::lazy_static;

mod character;
mod handlers;
mod map;
mod player;
mod settings;
mod world;
use settings::Settings;

mod handle_packet;
mod handle_player;
use handle_player::handle_player;

use std::{collections::HashMap, sync::Arc, time::Duration};

use eo::data::{EOByte, EOShort};
use mysql_async::prelude::*;
use player::Command;
use tokio::{
    net::TcpListener,
    sync::{mpsc, Mutex},
    time,
};

use world::World;

pub type PacketBuf = Vec<EOByte>;
pub type Tx = mpsc::UnboundedSender<Command>;
pub type Rx = mpsc::UnboundedReceiver<Command>;
pub type Players = Arc<Mutex<HashMap<EOShort, Tx>>>;

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

    lazy_static! {
        static ref SETTINGS: Settings = Settings::new().expect("Failed to load settings!");
    };

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
        let conn = pool.get_conn().await?;
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

    let mut world = World::new();
    world.load_maps(282).await?;
    world.load_pub_files().await?;

    let players: Players = Arc::new(Mutex::new(HashMap::new()));

    let listener =
        TcpListener::bind(format!("{}:{}", SETTINGS.server.host, SETTINGS.server.port)).await?;
    info!(
        "listening at {}:{}",
        SETTINGS.server.host, SETTINGS.server.port
    );

    let mut ping_interval = time::interval(Duration::from_secs(SETTINGS.server.ping_rate.into()));
    let ping_players = players.clone();
    // Skip the first tick because it's instant
    ping_interval.tick().await;
    tokio::spawn(async move {
        loop {
            ping_interval.tick().await;
            let mut players = ping_players.lock().await;
            for (_, tx) in players.iter_mut() {
                if let Err(e) = tx.send(Command::Ping) {
                    error!("there was an error sending ping: {:?}", e);
                }
            }
        }
    });

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        let players = players.clone();

        let num_of_players = players.lock().await.len();
        if num_of_players >= SETTINGS.server.max_connections as usize {
            warn!("{} has been disconnected because the server is full", addr);
            continue;
        }

        info!("connection accepted ({}) {}/{}", addr, num_of_players + 1, SETTINGS.server.max_connections);

        let pool = pool.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_player(players, socket, pool).await {
                error!("there was an error processing player: {:?}", e);
            }
        });
    }

    Ok(())
}
