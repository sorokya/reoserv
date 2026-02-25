use eolib::protocol::{
    net::{server::SitState, Item, Spell},
    AdminLevel, Direction, Gender,
};

use crate::{
    db::{insert_params, DbHandle},
    SETTINGS,
};

use super::{Character, QuestProgress};

impl Character {
    pub async fn load(
        db: &DbHandle,
        id: i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut character = Character::default();
        let row = match db
            .query_one(&insert_params(
                include_str!("../sql/get_character.sql"),
                &[("character_id", &id)],
            ))
            .await?
        {
            Some(row) => row,
            _ => {
                panic!(
                    "Attempting to load character that doesn't exist! ID: {}",
                    id
                );
            }
        };

        character.id = id;
        character.account_id = row.get_int(0).unwrap();
        character.name = row.get_string(1).unwrap();
        character.title = row.get_string(2);
        character.home = row.get_string(3).unwrap();
        character.fiance = row.get_string(4);
        character.partner = row.get_string(5);
        character.admin_level = AdminLevel::from(row.get_int(6).unwrap());
        character.class = row.get_int(7).unwrap();
        character.gender = Gender::from(row.get_int(8).unwrap());
        character.skin = row.get_int(9).unwrap();
        character.hair_style = row.get_int(10).unwrap();
        character.hair_color = row.get_int(11).unwrap();
        character.bank_level = row.get_int(12).unwrap();
        character.gold_bank = row.get_int(13).unwrap();
        character.guild_rank = row.get_int(14);
        character.guild_rank_string = row.get_string(15);
        character.equipment.boots = row.get_int(16).unwrap();
        character.equipment.accessory = row.get_int(17).unwrap();
        character.equipment.gloves = row.get_int(18).unwrap();
        character.equipment.belt = row.get_int(19).unwrap();
        character.equipment.armor = row.get_int(20).unwrap();
        character.equipment.hat = row.get_int(21).unwrap();
        character.equipment.shield = row.get_int(22).unwrap();
        character.equipment.weapon = row.get_int(23).unwrap();
        character.equipment.ring[0] = row.get_int(24).unwrap();
        character.equipment.ring[1] = row.get_int(25).unwrap();
        character.equipment.armlet[0] = row.get_int(26).unwrap();
        character.equipment.armlet[1] = row.get_int(27).unwrap();
        character.equipment.bracer[0] = row.get_int(28).unwrap();
        character.equipment.bracer[1] = row.get_int(29).unwrap();
        character.equipment.necklace = row.get_int(30).unwrap();
        character.level = row.get_int(31).unwrap();
        character.experience = row.get_int(32).unwrap();
        character.hp = row.get_int(33).unwrap();
        character.tp = row.get_int(34).unwrap();
        character.base_strength = row.get_int(35).unwrap();
        character.base_intelligence = row.get_int(36).unwrap();
        character.base_wisdom = row.get_int(37).unwrap();
        character.base_agility = row.get_int(38).unwrap();
        character.base_constitution = row.get_int(39).unwrap();
        character.base_charisma = row.get_int(40).unwrap();
        character.stat_points = row.get_int(41).unwrap();
        character.skill_points = row.get_int(42).unwrap();
        character.karma = row.get_int(43).unwrap();
        character.usage = row.get_int(44).unwrap();
        character.map_id = row.get_int(45).unwrap();
        character.coords.x = row.get_int(46).unwrap();
        character.coords.y = row.get_int(47).unwrap();
        character.direction = Direction::from(row.get_int(48).unwrap());
        character.sit_state = SitState::from(row.get_int(49).unwrap());
        character.hidden = row.get_int(50).unwrap() == 1;
        character.guild_name = row.get_string(51);
        character.guild_tag = row.get_string(52);

        character.items = db
            .query_map(
                &insert_params(
                    include_str!("../sql/get_character_inventory.sql"),
                    &[("character_id", &id)],
                ),
                |row| Item {
                    id: row.get_int(0).unwrap(),
                    amount: row.get_int(1).unwrap(),
                },
            )
            .await?;

        character.bank = db
            .query_map(
                &insert_params(
                    include_str!("../sql/get_character_bank.sql"),
                    &[("character_id", &id)],
                ),
                |row| Item {
                    id: row.get_int(0).unwrap(),
                    amount: row.get_int(1).unwrap(),
                },
            )
            .await?;

        character.spells = db
            .query_map(
                &insert_params(
                    include_str!("../sql/get_character_spells.sql"),
                    &[("character_id", &id)],
                ),
                |row| Spell {
                    id: row.get_int(0).unwrap(),
                    level: row.get_int(1).unwrap(),
                },
            )
            .await?;

        character.quests = db
            .query_map(
                &insert_params(
                    include_str!("../sql/get_character_quest_progress.sql"),
                    &[("character_id", &id)],
                ),
                |row| QuestProgress {
                    id: row.get_int(0).unwrap(),
                    state: row.get_int(1).unwrap(),
                    npc_kills: {
                        let json = row.get_string(2).unwrap();
                        match serde_json::from_str::<serde_json::Value>(&json) {
                            Ok(value) => match value.as_object() {
                                Some(object) => object
                                    .iter()
                                    .map(|(id, amount)| {
                                        (
                                            id.parse::<i32>().unwrap(),
                                            amount.as_i64().unwrap() as i32,
                                        )
                                    })
                                    .collect::<Vec<_>>(),
                                None => Vec::new(),
                            },
                            Err(_) => Vec::new(),
                        }
                    },
                    player_kills: row.get_int(3).unwrap(),
                    done_at: row.get_date(4),
                    completions: row.get_int(5).unwrap(),
                },
            )
            .await?;

        character.auto_pickup_items = db
            .query_map(
                &insert_params(
                    include_str!("../sql/get_character_auto_pickup.sql"),
                    &[("character_id", &id)],
                ),
                |row| row.get_int(0).unwrap(),
            )
            .await?;

        character.warp_suck_ticks = SETTINGS.world.warp_suck_rate;
        character.ghost_ticks = SETTINGS.world.ghost_rate;

        Ok(character)
    }
}
