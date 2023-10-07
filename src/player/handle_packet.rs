use super::{handlers, ClientState, PlayerHandle};
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

    match family {
        PacketFamily::Shop => match action {
            PacketAction::Buy => {
                handlers::shop::buy(reader, player.clone()).await;
            }
            PacketAction::Create => {
                handlers::shop::craft(reader, player.clone()).await;
            }
            PacketAction::Open => {
                handlers::shop::open(reader, player.clone()).await;
            }
            PacketAction::Sell => {
                handlers::shop::sell(reader, player.clone()).await;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Sit => match action {
            PacketAction::Request => {
                handlers::sit::request(reader, player.clone()).await;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Chest => match action {
            PacketAction::Open => {
                handlers::chest::open(reader, player.clone()).await;
            }
            PacketAction::Take => {
                handlers::chest::take(reader, player.clone()).await;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Paperdoll => match action {
            PacketAction::Add => {
                handlers::paperdoll::add(reader, player.clone()).await;
            }
            PacketAction::Remove => {
                handlers::paperdoll::remove(reader, player.clone()).await;
            }
            PacketAction::Request => {
                handlers::paperdoll::request(reader, player.clone()).await;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Item => match action {
            PacketAction::Drop => {
                handlers::item::drop(reader, player.clone()).await;
            }
            PacketAction::Get => {
                handlers::item::get(reader, player.clone()).await;
            }
            PacketAction::Junk => {
                handlers::item::junk(reader, player.clone()).await;
            }
            PacketAction::Use => {
                handlers::item::r#use(reader, player.clone()).await;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Attack => match action {
            PacketAction::Use => {
                handlers::attack::r#use(reader, player.clone()).await;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Talk => match action {
            PacketAction::Announce => {
                handlers::talk::announce(reader, player.clone(), world.clone()).await;
            }
            PacketAction::Tell => {
                handlers::talk::tell(reader, player.clone(), world.clone()).await;
            }
            PacketAction::Report => {
                handlers::talk::report(reader, player.clone(), world.clone()).await?;
            }
            PacketAction::Admin => {
                handlers::talk::admin(reader, player.clone(), world.clone()).await;
            }
            PacketAction::Msg => {
                handlers::talk::message(reader, player.clone(), world.clone()).await;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Init => match action {
            PacketAction::Init => {
                handlers::init::request(reader, player.clone()).await?;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Connection => match action {
            PacketAction::Accept => {
                handlers::connection::accept(reader, player.clone()).await?;
            }
            PacketAction::Ping => {
                player.pong();
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Account => match action {
            PacketAction::Request => {
                handlers::account::request(reader, player.clone(), world.clone()).await;
            }
            PacketAction::Create => {
                handlers::account::create(reader, player.clone(), world.clone()).await;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Login => match action {
            PacketAction::Request => {
                handlers::login::request(reader, player.clone(), world.clone()).await?;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Character => match action {
            PacketAction::Request => {
                handlers::character::request(reader, player.clone(), world.clone()).await;
            }
            PacketAction::Create => {
                handlers::character::create(reader, player.clone(), world.clone()).await;
            }
            PacketAction::Take => {
                handlers::character::take(reader, player.clone(), world.clone()).await;
            }
            PacketAction::Remove => {
                handlers::character::remove(reader, player.clone(), world.clone()).await;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Door => match action {
            PacketAction::Open => {
                handlers::door::open(reader, player.clone()).await?;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Emote => match action {
            PacketAction::Report => {
                handlers::emote::report(reader, player.clone()).await?;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Walk => match action {
            PacketAction::Player => {
                handlers::walk::player(reader, player.clone()).await?;
            }
            PacketAction::Spec => {
                handlers::walk::spec(reader, player.clone()).await?;
            }
            PacketAction::Admin => {
                handlers::walk::admin(reader, player.clone()).await?;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Players => match action {
            PacketAction::List | PacketAction::Request => {
                handlers::players::list(reader, player.clone(), world.clone()).await;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Welcome => match action {
            PacketAction::Request => {
                handlers::welcome::request(reader, player.clone(), world.clone()).await;
            }
            PacketAction::Agree => {
                handlers::welcome::agree(reader, player.clone(), world.clone()).await;
            }
            PacketAction::Msg => {
                handlers::welcome::message(reader, player.clone(), world.clone()).await;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Face => match action {
            PacketAction::Player => {
                handlers::face::player(reader, player.clone()).await?;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::PlayerRange => match action {
            PacketAction::Request => {
                handlers::character_map_info::request(reader, player.clone()).await;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Range => match action {
            PacketAction::Request => handlers::map_info::request(reader, player.clone()).await,
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Refresh => match action {
            PacketAction::Request => handlers::refresh::request(player.clone()).await?,
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::Warp => match action {
            PacketAction::Accept => handlers::warp::accept(reader, player.clone()).await,
            PacketAction::Take => handlers::warp::take(reader, player.clone(), world.clone()).await,
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },
        PacketFamily::NPCRange => match action {
            PacketAction::Request => handlers::npc_map_info::request(reader, player.clone()).await,
            _ => error!("Unhandles packet {:?}_{:?}", action, family),
        },
        _ => {
            error!("Unhandled packet {:?}_{:?}", action, family);
        }
    }

    player.set_busy(false);

    Ok(())
}
