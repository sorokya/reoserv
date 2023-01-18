use eo::{
    data::EOInt,
    protocol::{AdminLevel, CharacterInfo, Gender, PaperdollBahsw, Skin},
};
use mysql_async::{prelude::*, Conn, Row};

pub async fn get_character_list(
    conn: &mut Conn,
    account_id: EOInt,
) -> Result<Vec<CharacterInfo>, Box<dyn std::error::Error + Send + Sync>> {
    let characters = conn
        .exec_map(
            include_str!("../../sql/get_character_list.sql"),
            params! {
                "account_id" => &account_id,
            },
            |row: Row| CharacterInfo {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                level: row.get(2).unwrap(),
                gender: Gender::from_char(row.get(3).unwrap()).unwrap(),
                hairstyle: row.get(4).unwrap(),
                haircolor: row.get(5).unwrap(),
                skin: Skin::from_char(row.get(6).unwrap()).unwrap(),
                admin: AdminLevel::from_char(row.get(7).unwrap()).unwrap(),
                paperdoll: PaperdollBahsw {
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
