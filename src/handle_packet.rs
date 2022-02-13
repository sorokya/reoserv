use eo::{
    data::{EOInt, EOShort, StreamReader, MAX1},
    net::{Action, Family},
};
use lazy_static::lazy_static;
use mysql_async::Pool;
use num_traits::FromPrimitive;

use crate::{
    handlers,
    player::{Command, PacketBus},
    settings::Settings,
    PacketBuf, Players,
};

pub async fn handle_packet(
    player_id: EOShort,
    packet: PacketBuf,
    bus: &mut PacketBus,
    players: Players,
    db_pool: Pool,
) -> Result<(), Box<dyn std::error::Error>> {
    let action = Action::from_u8(packet[0]).unwrap();
    let family = Family::from_u8(packet[1]).unwrap();
    let reader = StreamReader::new(&packet[2..]);

    lazy_static! {
        static ref SETTINGS: Settings = Settings::new().expect("Failed to load settings!");
    };

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

        if SETTINGS.server.enforce_sequence && server_sequence != client_sequence {
            players
                .lock()
                .await
                .get(&player_id)
                .unwrap()
                .send(Command::Close(format!(
                    "sending invalid sequence: Got {}, expected {}.",
                    client_sequence, server_sequence
                )))?;
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
                    player_id,
                    bus.packet_processor.decode_multiple,
                    bus.packet_processor.encode_multiple,
                    players.lock().await.get(&player_id).unwrap(),
                )
                .await?;
            }
            Action::Ping => {
                players
                    .lock()
                    .await
                    .get(&player_id)
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
                if bus.sequencer.too_big_for_account_reply() {
                    bus.sequencer.account_reply_new_sequence();
                }
                handlers::account::request(
                    buf,
                    players.lock().await.get(&player_id).unwrap(),
                    &mut conn,
                    bus.sequencer.get_sequence_start(),
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
                    players.lock().await.get(&player_id).unwrap(),
                    &mut conn,
                    SETTINGS.server.password_salt.to_string(),
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
