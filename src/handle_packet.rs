use std::sync::Arc;

use eo::{
    data::{EOInt, StreamReader, MAX1},
    net::{Action, Family},
};
use lazy_static::lazy_static;
use mysql_async::Pool;
use num_traits::FromPrimitive;
use tokio::sync::Mutex;

use crate::{
    character::Character,
    handlers,
    player::{Command, Player},
    settings::Settings,
    world::World,
    PacketBuf, Players,
};

pub async fn handle_packet(
    packet: PacketBuf,
    player: &mut Player,
    world: Arc<Mutex<World>>,
    players: Players,
    active_account_ids: Arc<Mutex<Vec<u32>>>,
    db_pool: Pool,
    characters: Arc<Mutex<Vec<Character>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let action = Action::from_u8(packet[0]).unwrap();
    let family = Family::from_u8(packet[1]).unwrap();
    let reader = StreamReader::new(&packet[2..]);

    lazy_static! {
        static ref SETTINGS: Settings = Settings::new().expect("Failed to load settings!");
    };

    if family != Family::Init {
        if family == Family::Connection && action == Action::Ping {
            player.bus.sequencer.pong_new_sequence();
        }

        let server_sequence = player.bus.sequencer.gen_sequence();
        let client_sequence = if server_sequence >= MAX1 {
            reader.get_short() as EOInt
        } else {
            reader.get_char() as EOInt
        };

        if SETTINGS.server.enforce_sequence && server_sequence != client_sequence {
            players
                .lock()
                .await
                .get(&player.id)
                .unwrap()
                .send(Command::Close(format!(
                    "sending invalid sequence: Got {}, expected {}.",
                    client_sequence, server_sequence
                )))?;
        }
    } else {
        player.bus.sequencer.gen_sequence();
    }

    let buf = reader.get_vec(reader.remaining());
    match family {
        Family::Init => match action {
            Action::Init => {
                handlers::init::init(
                    buf,
                    player.id,
                    player.bus.sequencer.get_init_sequence_bytes(),
                    player.bus.packet_processor.decode_multiple,
                    player.bus.packet_processor.encode_multiple,
                    players.lock().await.get(&player.id).unwrap(),
                )
                .await?;
            }
            _ => {
                error!("Unhandled packet {:?}_{:?}", action, family);
            }
        },
        Family::Connection => match action {
            Action::Accept => {
                handlers::connection::accept(
                    buf,
                    player.id,
                    player.bus.packet_processor.decode_multiple,
                    player.bus.packet_processor.encode_multiple,
                    players.lock().await.get(&player.id).unwrap(),
                )
                .await?;
            }
            Action::Ping => {
                players
                    .lock()
                    .await
                    .get(&player.id)
                    .unwrap()
                    .send(Command::Pong)?;
            }
            _ => {
                error!("Unhandled packet {:?}_{:?}", action, family);
            }
        },
        Family::Account => match action {
            Action::Request => {
                let mut conn = db_pool.get_conn().await?;
                if player.bus.sequencer.too_big_for_account_reply() {
                    player.bus.sequencer.account_reply_new_sequence();
                }
                handlers::account::request(
                    buf,
                    players.lock().await.get(&player.id).unwrap(),
                    &mut conn,
                    player.bus.sequencer.get_sequence_start(),
                )
                .await?;
            }
            Action::Create => {
                let mut conn = db_pool.get_conn().await?;
                handlers::account::create(
                    buf,
                    players.lock().await.get(&player.id).unwrap(),
                    &mut conn,
                    SETTINGS.server.password_salt.to_string(),
                    &player.ip,
                )
                .await?;
            }
            _ => {
                error!("Unhandled packet {:?}_{:?}", action, family);
            }
        },
        Family::Login => match action {
            Action::Request => {
                let mut conn = db_pool.get_conn().await?;
                handlers::login::request(
                    buf,
                    players.lock().await.get(&player.id).unwrap(),
                    active_account_ids.lock().await.to_vec(),
                    &mut conn,
                    SETTINGS.server.password_salt.to_string(),
                )
                .await?;
            }
            _ => {
                error!("Unhandled packet {:?}_{:?}", action, family);
            }
        },
        Family::Character => match action {
            Action::Request => {
                handlers::character::request(
                    buf,
                    players.lock().await.get(&player.id).unwrap(),
                    player.num_of_characters,
                )
                .await?;
            }
            Action::Create => {
                let mut conn = db_pool.get_conn().await?;
                handlers::character::create(
                    buf,
                    players.lock().await.get(&player.id).unwrap(),
                    &mut conn,
                    player.account_id,
                )
                .await?;
            }
            Action::Take => {
                let mut conn = db_pool.get_conn().await?;
                handlers::character::take(
                    buf,
                    players.lock().await.get(&player.id).unwrap(),
                    &mut conn,
                    player.account_id,
                )
                .await?;
            }
            _ => {
                error!("Unhandled packet {:?}_{:?}", action, family);
            }
        },
        Family::Welcome => match action {
            Action::Request => {
                let mut conn = db_pool.get_conn().await?;
                handlers::welcome::request(
                    buf,
                    players.lock().await.get(&player.id).unwrap(),
                    &mut conn,
                    world,
                    player.id,
                    characters.clone(),
                )
                .await?;
            }
            Action::Agree => {
                let characters = characters.lock().await;
                let character = characters
                    .iter()
                    .find(|c| c.player_id == player.id)
                    .unwrap();
                handlers::welcome::agree(
                    buf,
                    players.lock().await.get(&player.id).unwrap(),
                    world,
                    character.map_id,
                )
                .await?;
            }
            Action::Message => {
                handlers::welcome::message(
                    buf,
                    players.lock().await.get(&player.id).unwrap(),
                    characters.clone(),
                    player.id,
                )
                .await?;
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
