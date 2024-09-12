use eolib::protocol::net::server::TalkServerServerPacket;
use eolib::protocol::net::{PacketAction, PacketFamily};

use crate::map::MapHandle;
use crate::player::PlayerHandle;
use crate::player_commands::{ArgType, Command};

use crate::{ITEM_DB, PLAYER_COMMANDS, SETTINGS};

#[derive(Debug, PartialEq, Eq)]
pub enum PlayerCommandResult {
    Ok,
    NotFound,
}

async fn autopickup(player_id: i32, args: &[String], player: &PlayerHandle, map: &MapHandle) {
    if !SETTINGS.auto_pickup.enabled {
        return;
    }

    let sub_command = match args.first() {
        Some(sub_command) => sub_command.as_str(),
        None => "list",
    };

    if sub_command == "list" {
        map.list_auto_pickup_items(player_id);
        return;
    }

    if sub_command == "clear" {
        map.clear_auto_pickup_items(player_id);
        return;
    }

    let identifier = (*args[1]).to_string();

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

    match sub_command {
        "add" => map.add_auto_pickup_item(player_id, item_id),
        "remove" => map.remove_auto_pickup_item(player_id, item_id),
        _ => {
            send_error_message(
                player,
                "Invalid argument. Must be \"list\", \"add\", or \"remove\".".to_string(),
            );
        }
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

pub async fn handle_player_command(
    player_id: i32,
    args: &[&str],
    player: &PlayerHandle,
    map: &MapHandle,
) -> PlayerCommandResult {
    let command = (*args[0]).to_string();
    let mut args: Vec<String> = args[1..].iter().map(|s| s.to_string()).collect();

    match PLAYER_COMMANDS
        .commands
        .iter()
        .find(|c| c.name == command || c.alias == command)
    {
        Some(command) => {
            if command.name.as_str() == "autopickup" && args.len() > 1 {
                let sub_command = args[0].to_string();
                let item_name = args[1..args.len()].join(" ");
                args = vec![sub_command, item_name];
            }

            if validate_args(&args, command, player) {
                match command.name.as_str() {
                    "autopickup" => autopickup(player_id, &args, player, map).await,
                    _ => {
                        return PlayerCommandResult::NotFound;
                    }
                }
            }
        }
        None => {
            return PlayerCommandResult::NotFound;
        }
    }

    PlayerCommandResult::Ok
}
