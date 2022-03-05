mod command;
pub use command::Command;
mod account_exists;
mod character_exists;
mod create_account;
mod create_character;
mod data;
mod delete_character;
mod get_character_list;
mod login;
mod world;
mod world_handle;
use eo::{
    data::EOShort,
    net::{CharacterInfo, CharacterList},
};
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
