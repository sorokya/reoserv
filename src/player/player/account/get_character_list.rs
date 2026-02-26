use eolib::protocol::{
    net::server::{CharacterSelectionListEntry, EquipmentCharacterSelect},
    AdminLevel, Gender,
};

use crate::{
    db::{insert_params, DbHandle},
    ITEM_DB,
};

pub async fn get_character_list(
    db: &DbHandle,
    account_id: i32,
) -> Result<Vec<CharacterSelectionListEntry>, Box<dyn std::error::Error + Send + Sync>> {
    let characters = db
        .query_map(
            &insert_params(
                include_str!("../../../sql/get_character_list.sql"),
                &[("account_id", &account_id)],
            ),
            |row| {
                let boots = row.get_int(8).unwrap();
                let armor = row.get_int(9).unwrap();
                let hat = row.get_int(10).unwrap();
                let shield = row.get_int(11).unwrap();
                let weapon = row.get_int(12).unwrap();

                CharacterSelectionListEntry {
                    id: row.get_int(0).unwrap(),
                    name: row.get_string(1).unwrap(),
                    level: row.get_int(2).unwrap(),
                    gender: Gender::from(row.get_int(3).unwrap()),
                    hair_style: row.get_int(4).unwrap(),
                    hair_color: row.get_int(5).unwrap(),
                    skin: row.get_int(6).unwrap(),
                    admin: AdminLevel::from(row.get_int(7).unwrap()),
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
