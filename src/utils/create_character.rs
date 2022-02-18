use eo::character::{Gender, Race};
use mysql_async::{prelude::*, Conn, TxOpts};

use lazy_static::lazy_static;

use crate::settings::Settings;

pub struct CreateCharacterParams {
    pub account_id: u32,
    pub name: String,
    pub gender: Gender,
    pub race: Race,
    pub hair_style: u32,
    pub hair_color: u32,
}

pub async fn create_character(
    conn: &mut Conn,
    params: CreateCharacterParams,
) -> Result<(), Box<dyn std::error::Error>> {
    lazy_static! {
        static ref SETTINGS: Settings = Settings::new().expect("Failed to load settings!");
    };

    let mut tx = conn.start_transaction(TxOpts::default()).await?;

    tx.exec_drop(
        include_str!("../sql/create_character.sql"),
        params! {
            "account_id" => &params.account_id,
            "name" => &params.name,
            "home" => &SETTINGS.new_character.home,
            "gender" => &(params.gender as u32),
            "race" => &(params.race as u32),
            "hair_style" => &params.hair_style,
            "hair_color" => &params.hair_color,
            "bank_max" => 0, // TODO: figure out bank max
        },
    )
    .await?;

    let character_id = tx.last_insert_id().unwrap();

    tx.exec_drop(
        r"INSERT INTO `Paperdoll` (
            `character_id`
        ) VALUES (:character_id);",
        params! {
            "character_id" => &character_id,
        },
    )
    .await?;

    tx.exec_drop(
        r"INSERT INTO `Position` (
            `character_id`,
            `map`,
            `x`,
            `y`,
            `direction`
        ) VALUES (
            :character_id,
            :map,
            :x,
            :y,
            :direction
        );",
        params! {
            "character_id" => &character_id,
            "map" => &SETTINGS.new_character.spawn_map,
            "x" => &SETTINGS.new_character.spawn_x,
            "y" => &SETTINGS.new_character.spawn_y,
            "direction" => &SETTINGS.new_character.spawn_direction,
        },
    )
    .await?;

    tx.exec_drop(
        r" INSERT INTO `Stats` (`character_id`)
        VALUES (:character_id);",
        params! {
            "character_id" => &character_id,
        },
    )
    .await?;

    tx.commit().await?;
    Ok(())
}
