use eo::data::{EOChar, EOShort, Serializeable};
use eo::net::packets::server::talk;
use eo::net::{Action, Family};
use eo::world::{TinyCoords, WarpAnimation};

use crate::commands::{ArgType, Command};
use crate::{character::Character, player::PlayerHandle, world::WorldHandle};

use crate::COMMANDS;

async fn warp(args: &[&str], character: &Character, world: &WorldHandle) {
    let map_id = args[0].parse::<EOShort>().unwrap();
    if let Ok(map) = world.get_map(map_id).await {
        let coords = if args.len() >= 3 {
            TinyCoords::new(
                args[1].parse::<EOChar>().unwrap(),
                args[2].parse::<EOChar>().unwrap(),
            )
        } else {
            let map_size = map.get_dimensions().await;
            TinyCoords::new(map_size.0 / 2, map_size.1 / 2)
        };

        character.player.as_ref().unwrap().request_warp(
            map_id,
            coords,
            false,
            Some(WarpAnimation::Admin),
        )
    } else {
        character.player.as_ref().unwrap().send(
            Action::Server,
            Family::Talk,
            talk::Server {
                message: format!("Map {} does not exist.", map_id),
            }
            .serialize(),
        );
    }
}

async fn warp_to_me(args: &[&str], character: &Character, world: &WorldHandle) {
    let target_name = (*args[0]).to_string();
    if let Ok(target) = world.get_character_by_name(target_name).await {
        target.player.as_ref().unwrap().request_warp(
            character.map_id,
            character.coords,
            false,
            Some(WarpAnimation::Admin),
        );
    }
}

async fn warp_me_to(args: &[&str], character: &Character, world: &WorldHandle) {
    let target_name = (*args[0]).to_string();
    if let Ok(target) = world.get_character_by_name(target_name).await {
        character.player.as_ref().unwrap().request_warp(
            target.map_id,
            target.coords,
            false,
            Some(WarpAnimation::Admin),
        );
    }
}

fn validate_args(args: &[&str], command: &Command, player: &PlayerHandle) -> bool {
    let required_args_length = command.args.iter().filter(|arg| arg.required).count();
    if args.len() < required_args_length {
        let packet = talk::Server {
            message: format!(
                "Wrong number of args. Got {}, expected: {}. (usage: \"{}\")",
                args.len(),
                required_args_length,
                command.usage
            ),
        };
        player.send(Action::Server, Family::Talk, packet.serialize());
        return false;
    }

    if args.len() > command.args.len() {
        let packet = talk::Server {
            message: format!(
                "Too many args. Got {}, expected: {}. (usage: \"{}\")",
                args.len(),
                command.args.len(),
                command.usage
            ),
        };
        player.send(Action::Server, Family::Talk, packet.serialize());
        return false;
    }

    for (i, raw_arg) in args.iter().enumerate() {
        let arg = &command.args[i];
        let valid_type = match arg.r#type {
            ArgType::UInt => raw_arg.parse::<u32>().is_ok(),
            ArgType::String => true,
            ArgType::Bool => raw_arg.parse::<bool>().is_ok(),
        };
        if !valid_type {
            let packet = talk::Server {
                message: format!(
                    "Invalid arg type. Got {}, expected: {:?}. (usage: \"{}\")",
                    raw_arg, arg.r#type, command.usage
                ),
            };
            player.send(Action::Server, Family::Talk, packet.serialize());
            return false;
        }
    }
    true
}

pub async fn handle_command(
    args: &[&str],
    character: &Character,
    player: PlayerHandle,
    world: WorldHandle,
) {
    let command = (*args[0]).to_string();
    let args = args[1..].to_vec();
    match COMMANDS
        .commands
        .iter()
        .find(|c| c.name == command || c.alias == command)
    {
        Some(command) => {
            if character.admin_level as EOChar >= command.admin_level as EOChar
                && validate_args(&args, command, &player)
            {
                match command.name.as_str() {
                    "warp" => warp(&args, character, &world).await,
                    "warptome" => warp_to_me(&args, character, &world).await,
                    "warpmeto" => warp_me_to(&args, character, &world).await,
                    _ => {
                        let packet = talk::Server {
                            message: format!("Unimplemented command: {}", command.name),
                        };
                        player.send(Action::Server, Family::Talk, packet.serialize());
                    }
                }
            }
        }
        None => {
            let packet = talk::Server {
                message: format!("Unknown command: {}", command),
            };
            player.send(Action::Server, Family::Talk, packet.serialize());
        }
    }
}