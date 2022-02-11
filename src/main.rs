const VERSION: &str = "0.0.0";

extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate config;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use num_traits::FromPrimitive;
use lazy_static::lazy_static;

mod character;
mod handlers;
mod map;
mod player;
mod settings;
mod world;
use settings::Settings;

use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::Duration,
};

use eo::{
    data::{EOByte, EOInt, EOShort, StreamReader, MAX1, StreamBuilder},
    net::{Action, Family},
};
use player::{Command, PacketBus, Player};
use tokio::{
    net::{TcpListener, TcpStream},
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

    lazy_static!(
        static ref SETTINGS: Settings = Settings::new().expect("Failed to load settings!");
    );

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

        if players.lock().await.len() >= SETTINGS.server.max_connections as usize {
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

    Ok(())
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
                        players.lock().await.remove(&player_id);
                        break;
                    }
                    Command::Ping => {
                        if player.bus.need_pong {
                            info!("player {} connection closed: ping timeout", player_id);
                            players.lock().await.remove(&player_id);
                            break;
                        } else {
                            player.bus.sequencer.ping_new_sequence();
                            let sequence = player.bus.sequencer.get_update_sequence_bytes();
                            let mut builder = StreamBuilder::with_capacity(3);
                            builder.add_short(sequence.0);
                            builder.add_char(sequence.1);
                            player.bus.need_pong = true;
                            player.bus.send(Action::Player, Family::Connection, builder.get()).await?;
                        }
                    },
                    Command::Pong => {
                        player.bus.need_pong = false;
                    },
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
            Action::Ping => {
                players.lock().await.get(&player_id).unwrap().send(Command::Pong).unwrap();
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
