use eo::{
    data::{EOInt, EOShort, StreamReader, MAX1},
    net::{Action, Family},
};
use lazy_static::lazy_static;
use num_traits::FromPrimitive;
use tokio::sync::{mpsc::UnboundedSender, oneshot};

use super::{command::Command, handlers};

use crate::{settings::Settings, world::WorldHandle, PacketBuf};

pub async fn handle_packet(
    packet: PacketBuf,
    player_id: EOShort,
    player: UnboundedSender<Command>,
    world: WorldHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let action = Action::from_u8(packet[0]).unwrap();
    let family = Family::from_u8(packet[1]).unwrap();
    let reader = StreamReader::new(&packet[2..]);

    lazy_static! {
        static ref SETTINGS: Settings = Settings::new().expect("Failed to load settings!");
    };

    if family != Family::Init {
        if family == Family::Connection && action == Action::Ping {
            let (tx, rx) = oneshot::channel();
            let _ = player.send(Command::PongNewSequence { respond_to: tx });
            rx.await.unwrap();
        }

        let server_sequence = {
            let (tx, rx) = oneshot::channel();
            let _ = player.send(Command::GenSequence { respond_to: tx });
            rx.await.unwrap()
        };

        let client_sequence = if server_sequence >= MAX1 {
            reader.get_short() as EOInt
        } else {
            reader.get_char() as EOInt
        };

        if SETTINGS.server.enforce_sequence && server_sequence != client_sequence {
            player.send(Command::Close(format!(
                "sending invalid sequence: Got {}, expected {}.",
                client_sequence, server_sequence
            )))?;
        }
    } else {
        let (tx, rx) = oneshot::channel();
        let _ = player.send(Command::GenSequence { respond_to: tx });
        rx.await.unwrap();
    }

    let buf = reader.get_vec(reader.remaining());
    match family {
        Family::Init => match action {
            Action::Init => {
                handlers::init::init(buf, player_id, player.clone()).await?;
            }
            _ => {
                error!("Unhandled packet {:?}_{:?}", action, family);
            }
        },
        Family::Connection => match action {
            Action::Accept => {
                handlers::connection::accept(buf, player_id, player.clone()).await?;
            }
            Action::Ping => {
                player.send(Command::Pong)?;
            }
            _ => {
                error!("Unhandled packet {:?}_{:?}", action, family);
            }
        },
        Family::Account => match action {
            Action::Request => {
                handlers::account::request(
                    buf,
                    player.clone(),
                    world.clone(),
                )
                .await?;
            }
            Action::Create => {
                handlers::account::create(
                    buf,
                    player.clone(),
                    world.clone(),
                )
                .await?;
            }
            _ => {
                error!("Unhandled packet {:?}_{:?}", action, family);
            }
        },
        //     Family::Login => match action {
        //         Action::Request => {
        //             let mut conn = db_pool.get_conn().await?;
        //             handlers::login::request(
        //                 buf,
        //                 players.lock().await.get(&player.id).unwrap(),
        //                 active_account_ids.lock().await.to_vec(),
        //                 &mut conn,
        //                 SETTINGS.server.password_salt.to_string(),
        //             )
        //             .await?;
        //         }
        //         _ => {
        //             error!("Unhandled packet {:?}_{:?}", action, family);
        //         }
        //     },
        //     Family::Character => match action {
        //         Action::Request => {
        //             handlers::character::request(
        //                 buf,
        //                 players.lock().await.get(&player.id).unwrap(),
        //                 player.num_of_characters,
        //             )
        //             .await?;
        //         }
        //         Action::Create => {
        //             let mut conn = db_pool.get_conn().await?;
        //             handlers::character::create(
        //                 buf,
        //                 players.lock().await.get(&player.id).unwrap(),
        //                 &mut conn,
        //                 player.account_id,
        //             )
        //             .await?;
        //         }
        //         Action::Take => {
        //             let mut conn = db_pool.get_conn().await?;
        //             handlers::character::take(
        //                 buf,
        //                 players.lock().await.get(&player.id).unwrap(),
        //                 &mut conn,
        //                 player.account_id,
        //             )
        //             .await?;
        //         }
        //         _ => {
        //             error!("Unhandled packet {:?}_{:?}", action, family);
        //         }
        //     },
        //     Family::Welcome => match action {
        //         Action::Request => {
        //             let mut conn = db_pool.get_conn().await?;
        //             handlers::welcome::request(
        //                 buf,
        //                 players.lock().await.get(&player.id).unwrap(),
        //                 &mut conn,
        //                 world,
        //                 player.id,
        //                 characters.clone(),
        //             )
        //             .await?;
        //         }
        //         Action::Agree => {
        //             let characters = characters.lock().await;
        //             let character = characters
        //                 .iter()
        //                 .find(|c| c.player_id == player.id)
        //                 .unwrap();
        //             handlers::welcome::agree(
        //                 buf,
        //                 players.lock().await.get(&player.id).unwrap(),
        //                 world,
        //                 character.map_id,
        //             )
        //             .await?;
        //         }
        //         Action::Message => {
        //             handlers::welcome::message(
        //                 buf,
        //                 players.lock().await.get(&player.id).unwrap(),
        //                 characters.clone(),
        //                 player.id,
        //             )
        //             .await?;
        //         }
        //         _ => {
        //             error!("Unhandled packet {:?}_{:?}", action, family);
        //         }
        //     },
        _ => {
            error!("Unhandled packet {:?}_{:?}", action, family);
        }
    }

    player.send(Command::SetBusy(false))?;

    Ok(())
}
