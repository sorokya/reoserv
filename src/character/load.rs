use eolib::protocol::{
    AdminLevel, Coords, Direction, Gender,
    net::{
        Item, Spell,
        server::{EquipmentPaperdoll, SitState},
    },
};

use crate::{
    SETTINGS,
    db::{DbHandle, Row, insert_params},
};

use super::{Character, QuestProgress};

impl Character {
    pub async fn load(db: &DbHandle, id: i32) -> anyhow::Result<Self> {
        let character_query = insert_params(
            include_str!("../sql/get_character.sql"),
            &[("character_id", &id)],
        );

        let inventory_query = insert_params(
            include_str!("../sql/get_character_inventory.sql"),
            &[("character_id", &id)],
        );

        let bank_query = insert_params(
            include_str!("../sql/get_character_bank.sql"),
            &[("character_id", &id)],
        );

        let spells_query = insert_params(
            include_str!("../sql/get_character_spells.sql"),
            &[("character_id", &id)],
        );

        let quest_progress_query = insert_params(
            include_str!("../sql/get_character_quest_progress.sql"),
            &[("character_id", &id)],
        );

        let auto_pickup_query = insert_params(
            include_str!("../sql/get_character_auto_pickup.sql"),
            &[("character_id", &id)],
        );

        let (row, items, bank, spells, quest_progress, auto_pickup) = match tokio::join!(
            db.query_one(&character_query),
            db.try_query_map(&inventory_query, |row| Ok(Item {
                id: row
                    .get_int(0)
                    .ok_or(anyhow::anyhow!("Failed to get item id"))?,
                amount: row
                    .get_int(1)
                    .ok_or(anyhow::anyhow!("Failed to get item amount"))?,
            }),),
            db.try_query_map(&bank_query, |row| Ok(Item {
                id: row
                    .get_int(0)
                    .ok_or(anyhow::anyhow!("Failed to get bank item id"))?,
                amount: row
                    .get_int(1)
                    .ok_or(anyhow::anyhow!("Failed to get bank item amount"))?,
            })),
            db.try_query_map(&spells_query, |row| Ok(Spell {
                id: row
                    .get_int(0)
                    .ok_or(anyhow::anyhow!("Failed to get spell id"))?,
                level: row
                    .get_int(1)
                    .ok_or(anyhow::anyhow!("Failed to get spell level"))?,
            })),
            db.try_query_map(&quest_progress_query, |row| Ok(QuestProgress {
                id: row
                    .get_int(0)
                    .ok_or(anyhow::anyhow!("Failed to get quest progress id"))?,
                state: row
                    .get_int(1)
                    .ok_or(anyhow::anyhow!("Failed to get quest progress state"))?,
                npc_kills: {
                    let json = row
                        .get_string(2)
                        .ok_or(anyhow::anyhow!("Failed to get quest progress npc kills"))?;
                    match serde_json::from_str::<serde_json::Value>(&json) {
                        Ok(value) => match value.as_object() {
                            Some(object) => object
                                .iter()
                                .filter_map(|(id, amount)| {
                                    Some((
                                        id.parse::<i32>().ok()?,
                                        amount.as_i64().map(|v| v as i32)?,
                                    ))
                                })
                                .collect::<Vec<_>>(),
                            None => Vec::new(),
                        },
                        Err(_) => Vec::new(),
                    }
                },
                player_kills: row
                    .get_int(3)
                    .ok_or(anyhow::anyhow!("Failed to get quest progress player kills"))?,
                done_at: row.get_date(4),
                completions: row
                    .get_int(5)
                    .ok_or(anyhow::anyhow!("Failed to get quest progress completions"))?,
            })),
            db.try_query_map(&auto_pickup_query, |row| row
                .get_int(0)
                .ok_or(anyhow::anyhow!("Failed to get auto pickup id")))
        ) {
            (
                Ok(Some(character)),
                Ok(items),
                Ok(bank),
                Ok(spells),
                Ok(quest_progress),
                Ok(auto_pickup),
            ) => (character, items, bank, spells, quest_progress, auto_pickup),
            (Err(e), _, _, _, _, _)
            | (_, Err(e), _, _, _, _)
            | (_, _, Err(e), _, _, _)
            | (_, _, _, Err(e), _, _)
            | (_, _, _, _, Err(e), _)
            | (_, _, _, _, _, Err(e)) => {
                return Err(anyhow::anyhow!(
                    "Failed to load character ID: {} data: {}",
                    id,
                    e
                ));
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Attempting to load character that doesn't exist! ID: {}",
                    id
                ));
            }
        };

        match Character::from_row(id, &row) {
            Some(mut character) => {
                character.items = items;
                character.bank = bank;
                character.spells = spells;
                character.quests = quest_progress;
                character.auto_pickup_items = auto_pickup;
                Ok(character)
            }
            None => Err(anyhow::anyhow!("Failed to parse character ID: {} data", id)),
        }
    }

    fn from_row(id: i32, row: &Row) -> Option<Self> {
        Some(Character {
            id,
            account_id: row.get_int(0)?,
            name: row.get_string(1)?,
            title: row.get_string(2),
            home: row.get_string(3)?,
            fiance: row.get_string(4),
            partner: row.get_string(5),
            admin_level: AdminLevel::from(row.get_int(6)?),
            class: row.get_int(7)?,
            gender: Gender::from(row.get_int(8)?),
            skin: row.get_int(9)?,
            hair_style: row.get_int(10)?,
            hair_color: row.get_int(11)?,
            bank_level: row.get_int(12)?,
            gold_bank: row.get_int(13)?,
            guild_rank: row.get_int(14),
            guild_rank_string: row.get_string(15),
            equipment: EquipmentPaperdoll {
                boots: row.get_int(16)?,
                accessory: row.get_int(17)?,
                gloves: row.get_int(18)?,
                belt: row.get_int(19)?,
                armor: row.get_int(20)?,
                hat: row.get_int(21)?,
                shield: row.get_int(22)?,
                weapon: row.get_int(23)?,
                ring: [row.get_int(24)?, row.get_int(25)?],
                armlet: [row.get_int(26)?, row.get_int(27)?],
                bracer: [row.get_int(28)?, row.get_int(29)?],
                necklace: row.get_int(30)?,
            },
            level: row.get_int(31)?,
            experience: row.get_int(32)?,
            hp: row.get_int(33)?,
            tp: row.get_int(34)?,
            base_strength: row.get_int(35)?,
            base_intelligence: row.get_int(36)?,
            base_wisdom: row.get_int(37)?,
            base_agility: row.get_int(38)?,
            base_constitution: row.get_int(39)?,
            base_charisma: row.get_int(40)?,
            stat_points: row.get_int(41)?,
            skill_points: row.get_int(42)?,
            karma: row.get_int(43)?,
            usage: row.get_int(44)?,
            map_id: row.get_int(45)?,
            coords: Coords {
                x: row.get_int(46)?,
                y: row.get_int(47)?,
            },
            direction: Direction::from(row.get_int(48)?),
            sit_state: SitState::from(row.get_int(49)?),
            hidden: row.get_int(50)? == 1,
            guild_name: row.get_string(51),
            guild_tag: row.get_string(52),
            warp_suck_ticks: SETTINGS.load().world.warp_suck_rate,
            ghost_ticks: SETTINGS.load().world.ghost_rate,
            ..Default::default()
        })
    }
}
