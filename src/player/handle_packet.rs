use super::{handlers, PlayerHandle, ClientState};
use bytes::Bytes;
use eo::{
    data::{EOInt, StreamReader},
    protocol::{PacketAction, PacketFamily},
};

use crate::{world::WorldHandle, SETTINGS};

pub async fn handle_packet(
    packet: Bytes,
    player: PlayerHandle,
    world: WorldHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let reader = StreamReader::new(packet);
    let action = PacketAction::from_byte(reader.get_byte()).unwrap();
    let family = PacketFamily::from_byte(reader.get_byte()).unwrap();

    if player.get_state().await? != ClientState::Uninitialized {
        if family != PacketFamily::Init {
            if family == PacketFamily::Connection && action == PacketAction::Ping {
                player.pong_new_sequence().await;
            }

            let server_sequence = player.gen_sequence().await?;
            let client_sequence = reader.get_char() as EOInt;

            if SETTINGS.server.enforce_sequence && server_sequence != client_sequence {
                player.close(format!(
                    "sending invalid sequence: Got {}, expected {}.",
                    client_sequence, server_sequence
                ));
            }
        } else {
            info!("{:?}_{:?}", family, action);
            player.gen_sequence().await?;
        }
    }

    let buf = reader.get_vec(reader.remaining());
    match family {
        PacketFamily::Item => match action {
            PacketAction::Get => {
                handlers::item::get(buf, player.clone()).await;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        }
        PacketFamily::Attack => match action {
            PacketAction::Use => {
                handlers::attack::r#use(buf, player.clone()).await;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        }
        PacketFamily::Talk => match action {
            PacketAction::Announce => {
                handlers::talk::announce(buf, player.clone(), world.clone()).await;
            }
            PacketAction::Tell => {
                handlers::talk::tell(buf, player.clone(), world.clone()).await;
            }
            PacketAction::Report => {
                handlers::talk::report(buf, player.clone(), world.clone()).await?;
            }
            PacketAction::Admin => {
                handlers::talk::admin(buf, player.clone(), world.clone()).await;
            }
            // PacketAction::Open => {
            //     handlers::talk::open(buf, player.clone(), world.clone()).await;
            // }
            // PacketAction::Request => {
            //     handlers::talk::request(buf, player.clone(), world.clone()).await;
            // }
            PacketAction::Msg => {
                handlers::talk::message(buf, player.clone(), world.clone()).await;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Init => match action {
            PacketAction::Init => {
                handlers::init::request(buf, player.clone()).await?;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Connection => match action {
            PacketAction::Accept => {
                handlers::connection::accept(buf, player.clone()).await?;
            }
            PacketAction::Ping => {
                player.pong();
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Account => match action {
            PacketAction::Request => {
                handlers::account::request(buf, player.clone(), world.clone()).await;
            }
            PacketAction::Create => {
                handlers::account::create(buf, player.clone(), world.clone()).await;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Login => match action {
            PacketAction::Request => {
                handlers::login::request(buf, player.clone(), world.clone()).await?;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Character => match action {
            PacketAction::Request => {
                handlers::character::request(buf, player.clone(), world.clone()).await;
            }
            PacketAction::Create => {
                handlers::character::create(buf, player.clone(), world.clone()).await;
            }
            PacketAction::Take => {
                handlers::character::take(buf, player.clone(), world.clone()).await;
            }
            PacketAction::Remove => {
                handlers::character::remove(buf, player.clone(), world.clone()).await;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Door => match action {
            PacketAction::Open => {
                handlers::door::open(buf, player.clone()).await?;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Emote => match action {
            PacketAction::Report => {
                handlers::emote::report(buf, player.clone()).await?;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Walk => match action {
            PacketAction::Player => {
                handlers::walk::player(buf, player.clone()).await?;
            }
            PacketAction::Spec => {
                handlers::walk::spec(buf, player.clone()).await?;
            }
            PacketAction::Admin => {
                handlers::walk::admin(buf, player.clone()).await?;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Players => match action {
            PacketAction::List | PacketAction::Request => {
                handlers::players::list(buf, player.clone(), world.clone()).await;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Welcome => match action {
            PacketAction::Request => {
                handlers::welcome::request(buf, player.clone(), world.clone()).await;
            }
            PacketAction::Agree => {
                handlers::welcome::agree(buf, player.clone(), world.clone()).await;
            }
            PacketAction::Msg => {
                handlers::welcome::message(buf, player.clone(), world.clone()).await;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Face => match action {
            PacketAction::Player => {
                handlers::face::player(buf, player.clone()).await?;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::PlayerRange => match action {
            PacketAction::Request => {
                handlers::character_map_info::request(buf, player.clone()).await;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Range => match action {
            PacketAction::Request => handlers::map_info::request(buf, player.clone()).await,
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Refresh => match action {
            PacketAction::Request => handlers::refresh::request(player.clone()).await?,
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Warp => match action {
            PacketAction::Accept => handlers::warp::accept(buf, player.clone()).await,
            PacketAction::Take => handlers::warp::take(buf, player.clone(), world.clone()).await,
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::NPCRange => match action {
            PacketAction::Request => handlers::npc_map_info::request(buf, player.clone()).await,
            _ => error!("Unhandles packet {:?}_{:?}", action, family),
        },
        _ => {
            error!("Unhandled packet {:?}_{:?}", action, family);
        }
    }

    player.set_busy(false);

    Ok(())
}
