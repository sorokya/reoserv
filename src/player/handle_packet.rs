use crate::{
    handlers,
    player::{ClientState, PlayerHandle},
};
use bytes::Bytes;
use eolib::{
    data::EoReader,
    protocol::net::{PacketAction, PacketFamily},
};

use crate::{world::WorldHandle, SETTINGS};

pub async fn handle_packet(
    packet: Bytes,
    player: PlayerHandle,
    world: WorldHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let reader = EoReader::new(packet);
    let action = PacketAction::from(reader.get_byte()?);
    if let PacketAction::Unrecognized(_) = action {
        player.close("invalid packet action".to_string());
        return Ok(());
    }

    let family = PacketFamily::from(reader.get_byte()?);
    if let PacketFamily::Unrecognized(_) = family {
        player.close("invalid packet family".to_string());
        return Ok(());
    }

    if player.get_state().await? != ClientState::Uninitialized {
        if family != PacketFamily::Init {
            if family == PacketFamily::Connection && action == PacketAction::Ping {
                player.pong_new_sequence().await;
            }

            let server_sequence = player.gen_sequence().await?;
            let client_sequence = reader.get_char()?;

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
        PacketFamily::Account => handlers::account(action, reader, player.clone()).await,
        PacketFamily::AdminInteract => {
            handlers::admin_interact(action, reader, player.clone(), world.clone()).await
        }
        PacketFamily::Attack => handlers::attack(action, reader, player.clone()).await,
        PacketFamily::Bank => handlers::bank(action, reader, player.clone()).await,
        PacketFamily::Board => handlers::board(action, reader, player.clone()).await,
        PacketFamily::Chair => handlers::chair(action, reader, player.clone()).await,
        PacketFamily::Character => handlers::character(action, reader, player.clone()).await,
        PacketFamily::Chest => handlers::chest(action, reader, player.clone()).await,
        PacketFamily::Citizen => handlers::citizen(action, reader, player.clone()).await,
        PacketFamily::Connection => handlers::connection(action, reader, player.clone()).await,
        PacketFamily::Door => handlers::door(action, reader, player.clone()).await,
        PacketFamily::Emote => handlers::emote(action, reader, player.clone()).await,
        PacketFamily::Face => handlers::face(action, reader, player.clone()).await,
        PacketFamily::Global => handlers::global(action, reader, player.clone()),
        PacketFamily::Init => handlers::init(action, reader, player.clone()).await,
        PacketFamily::Item => handlers::item(action, reader, player.clone()).await,
        PacketFamily::Jukebox => handlers::jukebox(action, reader, player.clone()).await,
        PacketFamily::Locker => handlers::locker(action, reader, player.clone()).await,
        PacketFamily::Login => handlers::login(action, reader, player.clone()).await,
        PacketFamily::Message => handlers::message(action, reader, player.clone()),
        PacketFamily::NpcRange => handlers::npc_range(action, reader, player.clone()).await,
        PacketFamily::Paperdoll => handlers::paperdoll(action, reader, player.clone()).await,
        PacketFamily::Party => handlers::party(action, reader, player.clone(), world.clone()).await,
        PacketFamily::PlayerRange => handlers::player_range(action, reader, player.clone()).await,
        PacketFamily::Players => {
            handlers::players(action, reader, player.clone(), world.clone()).await
        }
        PacketFamily::Range => handlers::range(action, reader, player.clone()).await,
        PacketFamily::Refresh => handlers::refresh(action, player.clone()).await,
        PacketFamily::Shop => handlers::shop(action, reader, player.clone()).await,
        PacketFamily::Sit => handlers::sit(action, reader, player.clone()).await,
        PacketFamily::Spell => handlers::spell(action, reader, player.clone()).await,
        PacketFamily::StatSkill => handlers::stat_skill(action, reader, player.clone()).await,
        PacketFamily::Talk => handlers::talk(action, reader, player.clone(), world.clone()).await,
        PacketFamily::Trade => handlers::trade(action, reader, player.clone()).await,
        PacketFamily::Walk => handlers::walk(reader, player.clone()).await,
        PacketFamily::Warp => handlers::warp(action, reader, player.clone()).await,
        PacketFamily::Welcome => handlers::welcome(action, reader, player.clone()).await,
        _ => {
            error!("Unhandled packet {:?}_{:?}", action, family);
        }
    }

    player.set_busy(false);

    Ok(())
}
