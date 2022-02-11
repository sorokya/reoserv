const VERSION: &str = "0.0.0";

extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate config;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use num_traits::FromPrimitive;

mod character;
mod handlers;
mod map;
mod player;
mod world;
mod settings;
use settings::Settings;

use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use eo::{
    data::{EOByte, EOInt, EOShort, StreamReader, MAX1},
    net::{Action, Family},
};
use player::{Command, PacketBus, Player};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};
use world::World;

pub type PacketBuf = Vec<EOByte>;
pub type Tx = mpsc::UnboundedSender<Command>;
pub type Rx = mpsc::UnboundedReceiver<Command>;
pub type Players = Arc<Mutex<HashMap<EOShort, Tx>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let settings = match Settings::new() {
        Ok(settings) => settings,
        _ => panic!("Failed to load settings!"),
    };

    let mut world = World::new();
    world.load_maps(282).await?;
    world.load_pub_files().await?;

    let players: Players = Arc::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind(format!("{}:{}", settings.server.host, settings.server.port)).await?;
    info!("listening at {}:{}", settings.server.host, settings.server.port);

    loop {
        let (socket, addr) = listener.accept().await?;
        let players = players.clone();

        if players.lock().await.len() >= settings.server.max_connections as usize {
            warn!("{} has been disconnected because the server is full", addr);
            continue;
        }

        info!("connection accepted ({})", addr);

        tokio::spawn(async move {
            if let Err(e) = handle_player(players, socket).await {
                error!("there was an error processing player: {:?}", e);
            }
        });
    }
}

async fn handle_player(
    players: Players,
    socket: TcpStream,
) -> Result<(), Box<dyn std::error::Error>> {
    let player_id = {
        let players = players.lock().await;
        get_next_player_id(&players, 1)
    };

    let mut player = Player::new(players.clone(), socket, player_id).await;
    let mut queue: RefCell<VecDeque<PacketBuf>> = RefCell::new(VecDeque::new());
    loop {
        tokio::select! {
            result = player.bus.recv() => match result {
                Some(Ok(packet)) => {
                    debug!("Recv: {:?}", packet);
                    queue.get_mut().push_back(packet);
                },
                Some(Err(e)) => {
                    error!("error receiving packet: {:?}", e);
                },
                None => {
                }
            },
            Some(command) = player.rx.recv() => {
                match command {
                    Command::Send(action, family, data) => {
                        player.bus.send(action, family, data).await?;
                    },
                    Command::Close(reason) => {
                        info!("player {} connection closed: {}", player_id, reason);
                        break;
                    }
                    _ => {
                        error!("unhandled command: {:?}", command);
                    }
                }
            },
        }

        if let Some(packet) = queue.get_mut().pop_front() {
            match handle_packet(player_id, packet, &mut player.bus, players.clone()).await {
                Ok(()) => {}
                Err(e) => {
                    error!("error handling packet: {:?}", e);
                }
            }
        }
    }

    Ok(())
}

async fn handle_packet(
    player_id: EOShort,
    packet: PacketBuf,
    bus: &mut PacketBus,
    players: Players,
) -> std::io::Result<()> {
    let action = Action::from_u8(packet[0]).unwrap();
    let family = Family::from_u8(packet[1]).unwrap();
    let reader = StreamReader::new(&packet[2..]);

    if family != Family::Init {
        if family == Family::Connection && action == Action::Ping {
            bus.sequencer.pong_new_sequence();
        }

        let server_sequence = bus.sequencer.gen_sequence();
        let client_sequence = if server_sequence > MAX1 {
            reader.get_short() as EOInt
        } else {
            reader.get_char() as EOInt
        };

        if server_sequence != client_sequence {
            players
                .lock()
                .await
                .get(&player_id)
                .unwrap()
                .send(Command::Close(format!(
                    "sending invalid sequence: Got {}, expected {}.",
                    client_sequence, server_sequence
                )))
                .unwrap();
        }
    } else {
        bus.sequencer.gen_sequence();
    }

    let buf = reader.get_vec(reader.remaining());

    match family {
        Family::Init => match action {
            Action::Init => {
                handlers::init::init(
                    buf,
                    player_id,
                    bus.sequencer.get_init_sequence_bytes(),
                    bus.packet_processor.decode_multiple,
                    bus.packet_processor.encode_multiple,
                    players.lock().await.get(&player_id).unwrap(),
                )
                .await
                .unwrap();
            }
            _ => {
                error!("Unhandled packet {:?}_{:?}", action, family);
            }
        },
        Family::Connection => match action {
            Action::Accept => {
                handlers::connection::accept(
                    buf,
                    player_id,
                    bus.packet_processor.decode_multiple,
                    bus.packet_processor.encode_multiple,
                    players.lock().await.get(&player_id).unwrap(),
                )
                .await
                .unwrap();
            }
            _ => {
                error!("Unhandled packet {:?}_{:?}", action, family);
            }
        },
        _ => {
            error!("Unhandled packet {:?}_{:?}", action, family);
        }
    }

    Ok(())
}

fn get_next_player_id(
    players: &tokio::sync::MutexGuard<HashMap<EOShort, Tx>>,
    seed: EOShort,
) -> EOShort {
    let id = seed;
    for player_id in players.iter().map(|c| *c.0) {
        if player_id == id {
            return get_next_player_id(players, id + 1);
        }
    }
    id
}
