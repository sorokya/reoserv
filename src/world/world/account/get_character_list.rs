use eolib::protocol::{net::server::{CharacterSelectionListEntry, EquipmentCharacterSelect}, Gender, AdminLevel};
use mysql_async::{prelude::*, Conn, Row};

use crate::ITEM_DB;

pub async fn get_character_list(
    conn: &mut Conn,
    account_id: i32,
) -> Result<Vec<CharacterSelectionListEntry>, Box<dyn std::error::Error + Send + Sync>> {
    let characters = conn
        .exec_map(
            include_str!("../../../sql/get_character_list.sql"),
            params! {
                "account_id" => &account_id,
            },
            |row: Row| {
                let boots: i32 = row.get(8).unwrap();
                let armor: i32 = row.get(9).unwrap();
                let hat: i32 = row.get(10).unwrap();
                let shield: i32 = row.get(11).unwrap();
                let weapon: i32 = row.get(12).unwrap();

                CharacterSelectionListEntry {
                    id: row.get(0).unwrap(),
                    name: row.get(1).unwrap(),
                    level: row.get(2).unwrap(),
                    gender: Gender::from(row.get::<i32, usize>(3).unwrap()),
                    hair_style: row.get(4).unwrap(),
                    hair_color: row.get(5).unwrap(),
                    skin: row.get(6).unwrap(),
                    admin: AdminLevel::from(row.get::<i32, usize>(7).unwrap()),
                    equipment: EquipmentCharacterSelect {
                        boots: match boots {
                            0 => 0,
                            _ => match ITEM_DB.items.get(boots as usize - 1) {
                                Some(item) => item.spec1,
                                None => 0,
                            },
                        },
                        armor: match armor {
                            0 => 0,
                            _ => match ITEM_DB.items.get(armor as usize - 1) {
                                Some(item) => item.spec1,
                                None => 0,
                            },
                        },
                        hat: match hat {
                            0 => 0,
                            _ => match ITEM_DB.items.get(hat as usize - 1) {
                                Some(item) => item.spec1,
                                None => 0,
                            },
                        },
                        shield: match shield {
                            0 => 0,
                            _ => match ITEM_DB.items.get(shield as usize - 1) {
                                Some(item) => item.spec1,
                                None => 0,
                            },
                        },
                        weapon: match weapon {
                            0 => 0,
                            _ => match ITEM_DB.items.get(weapon as usize - 1) {
                                Some(item) => item.spec1,
                                None => 0,
                            },
                        },
                    },
                }
            },
        )
        .await?;

    Ok(characters)
}
