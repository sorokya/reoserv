use eolib::protocol::net::server::{TalkServerServerPacket, WarpEffect};
use eolib::protocol::net::{PacketAction, PacketFamily};
use eolib::protocol::Coords;

use crate::commands::{ArgType, Command};
use crate::{character::Character, player::PlayerHandle, world::WorldHandle};

use crate::{COMMANDS, ITEM_DB, NPC_DB};

async fn warp(args: &[String], character: &Character, world: &WorldHandle) {
    let player = match character.player.as_ref() {
        Some(player) => player,
        None => return,
    };

    let map_id = args[0].parse::<i32>().unwrap();

    if let Ok(map) = world.get_map(map_id).await {
        let coords = if args.len() >= 3 {
            Coords {
                x: args[1].parse::<i32>().unwrap(),
                y: args[2].parse::<i32>().unwrap(),
            }
        } else {
            let map_size = map.get_dimensions().await;
            Coords {
                x: map_size.x / 2,
                y: map_size.y / 2,
            }
        };

        player.request_warp(map_id, coords, false, Some(WarpEffect::Admin))
    } else {
        send_error_message(player, format!("Map {} does not exist.", map_id));
    }
}

// TODO: warp player to where you're facing
async fn warp_to_me(args: &[String], character: &Character, world: &WorldHandle) {
    let target_name = &args[0];

    if let Ok(target) = world.get_character_by_name(target_name).await {
        let target = match target.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        target.request_warp(
            character.map_id,
            character.coords,
            false,
            Some(WarpEffect::Admin),
        );
    }
}

// TODO: Make light guide and guardian warp near player out of site with scroll warp
async fn warp_me_to(args: &[String], character: &Character, world: &WorldHandle) {
    let player = match character.player.as_ref() {
        Some(player) => player,
        None => return,
    };

    let target_name = &args[0];

    if let Ok(target) = world.get_character_by_name(target_name).await {
        player.request_warp(target.map_id, target.coords, false, Some(WarpEffect::Admin));
    }
}

async fn evacuate(character: &Character, world: &WorldHandle) {
    let map = match world.get_map(character.map_id).await {
        Ok(map) => map,
        Err(e) => {
            error!("Failed to get map: {}", e);
            return;
        }
    };

    map.start_evacuate();
}

async fn spawn_item(args: &[String], character: &Character) {
    let player = match character.player.as_ref() {
        Some(player) => player,
        None => return,
    };

    let identifier = (*args[0]).to_string();

    let item_id = match identifier.parse::<u32>() {
        Ok(id) => id as i32,
        Err(_) => {
            // find matches from item db where name starts with identifier
            match ITEM_DB
                .items
                .iter()
                .position(|item| item.name.to_lowercase() == identifier.to_lowercase())
            {
                Some(index) => index as i32 + 1,
                None => {
                    send_error_message(
                        player,
                        format!("No item found with name \"{}\".", identifier),
                    );
                    return;
                }
            }
        }
    };

    let amount = if args.len() >= 2 {
        args[1].parse::<i32>().unwrap()
    } else {
        1
    };

    if let Ok(map) = player.get_map().await {
        let player_id = match player.get_player_id().await {
            Ok(player_id) => player_id,
            Err(e) => {
                error!("Failed to get player id: {}", e);
                return;
            }
        };

        map.give_item(player_id, item_id, amount);
    }
}

async fn spawn_npc(args: &[String], character: &Character) {
    let player = match character.player.as_ref() {
        Some(player) => player,
        None => return,
    };

    let identifier = (*args[0]).to_string();

    let npc_id = match identifier.parse::<u32>() {
        Ok(id) => id as i32,
        Err(_) => {
            // find matches from item db where name starts with identifier
            match NPC_DB
                .npcs
                .iter()
                .position(|npc| npc.name.to_lowercase() == identifier.to_lowercase())
            {
                Some(index) => index as i32 + 1,
                None => {
                    send_error_message(
                        player,
                        format!("No npc found with name \"{}\".", identifier),
                    );
                    return;
                }
            }
        }
    };

    let amount = if args.len() >= 2 {
        args[1].parse::<i32>().unwrap()
    } else {
        1
    };

    let speed = if args.len() >= 3 {
        args[2].parse::<i32>().unwrap()
    } else {
        3
    };

    if let Ok(map) = player.get_map().await {
        let player_id = match player.get_player_id().await {
            Ok(player_id) => player_id,
            Err(e) => {
                error!("Failed to get player id: {}", e);
                return;
            }
        };

        map.spawn_npc(player_id, npc_id, amount, speed);
    }
}

async fn hide(character: &Character) {
    let player = match character.player.as_ref() {
        Some(player) => player,
        None => {
            return;
        }
    };

    let player_id = match player.get_player_id().await {
        Ok(player_id) => player_id,
        Err(e) => {
            error!("Failed to get player id: {}", e);
            return;
        }
    };

    if let Ok(map) = player.get_map().await {
        map.toggle_hidden(player_id);
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
            send_error_message(
                player,
                format!(
                    "Invalid arg type. Got {}, expected: {:?}. (usage: \"{}\")",
                    raw_arg, arg.r#type, command.usage
                ),
            );
            return false;
        }
    }
    true
}

fn send_error_message(player: &PlayerHandle, message: String) {
    player.send(
        PacketAction::Server,
        PacketFamily::Talk,
        &TalkServerServerPacket { message },
    );
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

            if i32::from(character.admin_level) >= i32::from(command.admin_level)
                && validate_args(&args, command, &player)
            {
                match command.name.as_str() {
                    "hide" => hide(character).await,
                    "spawnitem" => spawn_item(&args, character).await,
                    "spawnnpc" => spawn_npc(&args, character).await,
                    "warp" => warp(&args, character, &world).await,
                    "warptome" => warp_to_me(&args, character, &world).await,
                    "warpmeto" => warp_me_to(&args, character, &world).await,
                    "evacuate" => evacuate(character, &world).await,
                    "jail" => world.jail_player(args[0].to_owned(), character.name.to_owned()),
                    "free" => world.free_player(args[0].to_owned()),
                    "kick" => {
                        world.kick_player(args[0].to_owned(), character.name.to_owned(), false)
                    }
                    "skick" => {
                        world.kick_player(args[0].to_owned(), character.name.to_owned(), true)
                    }
                    "ban" => world.ban_player(
                        args[0].to_owned(),
                        if args.len() > 1 {
                            args[1].to_owned()
                        } else {
                            "".to_string()
                        },
                        character.name.to_owned(),
                        false,
                    ),
                    "sban" => world.ban_player(
                        args[0].to_owned(),
                        if args.len() > 1 {
                            args[1].to_owned()
                        } else {
                            "".to_string()
                        },
                        character.name.to_owned(),
                        true,
                    ),
                    "quake" => world.quake(if args.is_empty() {
                        1
                    } else {
                        args[0].parse::<i32>().unwrap()
                    }),
                    "mute" => world.mute_player(args[0].to_owned(), character.name.to_owned()),
                    "player" => {
                        world.request_player_info(character.player_id.unwrap(), args[0].to_owned())
                    }
                    "inventory" => world
                        .request_player_inventory(character.player_id.unwrap(), args[0].to_owned()),
                    "freeze" => world.freeze_player(args[0].to_owned(), character.name.to_owned()),
                    "unfreeze" => {
                        world.unfreeze_player(args[0].to_owned(), character.name.to_owned())
                    }
                    "global" => world.toggle_global(character.name.to_owned()),
                    "remap" => world.reload_map(character.map_id),
                    "captcha" => {
                        world.show_captcha(args[0].to_owned(), args[1].parse::<i32>().unwrap())
                    }
                    _ => {
                        send_error_message(
                            &player,
                            format!("Unimplemented command: {}", command.name),
                        );
                    }
                }
            }
        }
        None => {
            send_error_message(&player, format!("Unknown command: {}", command));
        }
    }
}
