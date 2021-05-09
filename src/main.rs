extern crate pretty_env_logger;
#[macro_use] extern crate log;

mod character;
mod map;
mod world;
mod player;
use std::{collections::HashMap, sync::{Arc, Mutex}};

use player::Player;
use eo::data::{EOByte, EOShort};
use tokio::{net::{TcpListener, TcpStream}, sync::mpsc};
use world::World;

pub type PacketBuf = Vec<EOByte>;
pub type Tx = mpsc::UnboundedSender<PacketBuf>;
pub type Rx = mpsc::UnboundedReceiver<PacketBuf>;
pub type Players = Arc<Mutex<HashMap<EOShort, Tx>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    println!("__________
\\______   \\ ____  ____  ______ ______________  __
 |       _// __ \\/  _ \\/  ___// __ \\_  __ \\  \\/ /
 |    |   \\  ___(  <_> )___ \\\\  ___/|  | \\/\\   /
 |____|_  /\\___  >____/____  >\\___  >__|    \\_/
        \\/     \\/          \\/     \\/\nThe rusty endless online server: v0.0.0\n");
    let mut world = World::new();
    world.load_maps(282).await?;
    world.load_pub_files().await?;

    let players: Players = Arc::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind("0.0.0.0:8078").await?;
    info!("listening at 0.0.0.0:8078");

    // tokio::spawn(async move {
    loop {
        let (socket, addr) = listener.accept().await?;
        let players = players.clone();
        info!("connection accepted ({})", addr);

        tokio::spawn(async move {
            if let Err(e) = accept_player(players, socket).await {
                error!("there was an error processing player: {:?}", e);
            }
        });
    }
    // });
}

async fn accept_player(players: Players, socket: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let player_id = {
        let players = players.lock().expect("Failed to lock players");
        get_next_player_id(&players, 1)
    };

    let mut player = Player::new(players.clone(), socket, player_id);
    loop {
        player.tick().await?;
        // TODO: break loop on disconnect
    }
    Ok(())
}

fn get_next_player_id(players: &std::sync::MutexGuard<HashMap<EOShort, Tx>>, seed: EOShort) -> EOShort {
    let id = seed;
    for player_id in players.iter().map(|c| *c.0) {
        if player_id == id {
            return get_next_player_id(players, id + 1);
        }
    }
    id
}


