use eo::data::{EOChar, EOShort, Serializeable, StreamBuilder};
use eo::protocol::server::talk;
use eo::protocol::{Coords, PacketAction, PacketFamily, WarpAnimation};

use crate::commands::{ArgType, Command};
use crate::{character::Character, player::PlayerHandle, world::WorldHandle};

use crate::{COMMANDS, ITEM_DB};

async fn warp(args: &[String], character: &Character, world: &WorldHandle) {
    let map_id = args[0].parse::<EOShort>().unwrap();
    if let Ok(map) = world.get_map(map_id).await {
        let coords = if args.len() >= 3 {
            Coords {
                x: args[1].parse::<EOChar>().unwrap(),
                y: args[2].parse::<EOChar>().unwrap(),
            }
        } else {
            let map_size = map.get_dimensions().await;
            Coords {
                x: map_size.0 / 2,
                y: map_size.1 / 2,
            }
        };

        character.player.as_ref().unwrap().request_warp(
            map_id,
            coords,
            false,
            Some(WarpAnimation::Admin),
        )
    } else {
        let packet = talk::Server {
            message: format!("Map {} does not exist.", map_id),
        };
        let mut builder = StreamBuilder::new();
        packet.serialize(&mut builder);
        character.player.as_ref().unwrap().send(
            PacketAction::Server,
            PacketFamily::Talk,
            builder.get(),
        );
    }
}

async fn warp_to_me(args: &[String], character: &Character, world: &WorldHandle) {
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

async fn warp_me_to(args: &[String], character: &Character, world: &WorldHandle) {
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

async fn spawn_item(args: &[String], character: &Character) {
    let identifier = (*args[0]).to_string();

    let item_id = match identifier.parse::<u32>() {
        Ok(id) => id as EOShort,
        Err(_) => {
            // find matches from item db where name starts with identifier
            match ITEM_DB
                .items
                .iter()
                .position(|item| item.name.to_lowercase() == identifier.to_lowercase())
            {
                Some(index) => index as EOShort + 1,
                None => {
                    let packet = talk::Server {
                        message: format!("No item found with name \"{}\".", identifier),
                    };
                    let mut builder = StreamBuilder::new();
                    packet.serialize(&mut builder);
                    character.player.as_ref().unwrap().send(
                        PacketAction::Server,
                        PacketFamily::Talk,
                        builder.get(),
                    );
                    return;
                }
            }
        }
    };

    let amount = if args.len() >= 2 {
        args[1].parse::<u32>().unwrap()
    } else {
        1
    };

    if let Ok(map) = character.player.as_ref().unwrap().get_map().await {
        let target_player_id = character
            .player
            .as_ref()
            .unwrap()
            .get_player_id()
            .await
            .unwrap();
        map.give_item(target_player_id, item_id, amount);
    }
}

fn validate_args(args: &[String], command: &Command, player: &PlayerHandle) -> bool {
    let required_args_length = command.args.iter().filter(|arg| arg.required).count();
    if args.len() < required_args_length {
        send_error_message(
            player,
            format!(
                "Wrong number of args. Got {}, expected: {}. (usage: \"{}\")",
                args.len(),
                required_args_length,
                command.usage
            ),
        );
        return false;
    }

    if args.len() > command.args.len() {
        send_error_message(
            player,
            format!(
                "Too many args. Got {}, expected: {}. (usage: \"{}\")",
                args.len(),
                command.args.len(),
                command.usage
            ),
        );
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
            let mut builder = StreamBuilder::new();
            packet.serialize(&mut builder);
            player.send(PacketAction::Server, PacketFamily::Talk, builder.get());
            return false;
        }
    }
    true
}

fn send_error_message(player: &PlayerHandle, message: String) {
    let packet = talk::Server { message };
    let mut builder = StreamBuilder::new();
    packet.serialize(&mut builder);
    player.send(PacketAction::Server, PacketFamily::Talk, builder.get());
}

pub async fn handle_command(
    args: &[&str],
    character: &Character,
    player: PlayerHandle,
    world: WorldHandle,
) {
    let command = (*args[0]).to_string();
    let mut args: Vec<String> = args[1..].iter().map(|s| s.to_string()).collect();

    match COMMANDS
        .commands
        .iter()
        .find(|c| c.name == command || c.alias == command)
    {
        Some(command) => {
            if command.name.as_str() == "spawnitem" && args.len() > 1 {
                if let Ok(amount) = args.last().unwrap().parse::<u32>() {
                    // join all but the last arg into a single string
                    let item_name = args[..args.len() - 1].join(" ");
                    args = vec![item_name, amount.to_string()];
                } else {
                    let item_name = args[..args.len()].join(" ");
                    args = vec![item_name];
                }
            }

            if character.admin_level as EOChar >= command.admin_level as EOChar
                && validate_args(&args, command, &player)
            {
                match command.name.as_str() {
                    "warp" => warp(&args, character, &world).await,
                    "warptome" => warp_to_me(&args, character, &world).await,
                    "warpmeto" => warp_me_to(&args, character, &world).await,
                    "spawnitem" => spawn_item(&args, character).await,
                    _ => {
                        let packet = talk::Server {
                            message: format!("Unimplemented command: {}", command.name),
                        };
                        let mut builder = StreamBuilder::new();
                        packet.serialize(&mut builder);
                        player.send(PacketAction::Server, PacketFamily::Talk, builder.get());
                    }
                }
            }
        }
        None => {
            let packet = talk::Server {
                message: format!("Unknown command: {}", command),
            };
            let mut builder = StreamBuilder::new();
            packet.serialize(&mut builder);
            player.send(PacketAction::Server, PacketFamily::Talk, builder.get());
        }
    }
}
