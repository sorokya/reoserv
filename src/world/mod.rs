mod command;
pub use command::Command;
mod create_account;
mod character_exists;
mod create_character;
mod delete_character;
mod account_exists;
mod get_character_list;
mod data;
mod login;
mod world;
mod world_handle;
use eo::{net::{CharacterInfo, CharacterList}, data::EOShort};
pub use world_handle::WorldHandle;

#[derive(Debug)]
pub enum LoginResult {
    Success {
        account_id: EOShort,
        character_list: CharacterList,
    },
    LoggedIn,
    WrongUsername,
    WrongPassword,
    Err(Box<dyn std::error::Error + Send + Sync>),
}