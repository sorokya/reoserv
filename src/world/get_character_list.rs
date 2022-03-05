use eo::{
    character::{AdminLevel, Gender, Race},
    data::EOShort,
    net::{CharacterInfo, PaperdollBAHSW},
};
use mysql_async::{prelude::*, Conn, Row};
use num_traits::FromPrimitive;

pub async fn get_character_list(
    conn: &mut Conn,
    account_id: EOShort,
) -> Result<Vec<CharacterInfo>, Box<dyn std::error::Error + Send + Sync>> {
    let characters = conn
        .exec_map(
            include_str!("../sql/get_character_list.sql"),
            params! {
                "account_id" => &account_id,
            },
            |row: Row| CharacterInfo {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                level: row.get(2).unwrap(),
                gender: Gender::from_u8(row.get(3).unwrap()).unwrap(),
                hair_style: row.get(4).unwrap(),
                hair_color: row.get(5).unwrap(),
                race: Race::from_u8(row.get(6).unwrap()).unwrap(),
                admin_level: AdminLevel::from_u8(row.get(7).unwrap()).unwrap(),
                paperdoll: PaperdollBAHSW {
                    boots: row.get(8).unwrap(),
                    armor: row.get(9).unwrap(),
                    hat: row.get(10).unwrap(),
                    shield: row.get(11).unwrap(),
                    weapon: row.get(12).unwrap(),
                },
            },
        )
        .await?;

    Ok(characters)
}
