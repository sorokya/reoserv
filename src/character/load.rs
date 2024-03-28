use chrono::{NaiveDateTime, TimeZone, Utc};
use eolib::protocol::{
    net::{server::SitState, Item, Spell},
    AdminLevel, Direction, Gender,
};
use mysql_async::{prelude::*, Conn, Params, Row};

use super::{Character, QuestProgress};

impl Character {
    pub async fn load(
        conn: &mut Conn,
        id: i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut character = Character::default();
        let mut row = match conn
            .exec_first::<Row, &str, Params>(
                include_str!("../sql/get_character.sql"),
                params! {
                    "character_id" => id,
                },
            )
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
        character.account_id = row.take("account_id").unwrap();
        character.name = row.take("name").unwrap();
        character.title = row.take("title").unwrap();
        character.home = row.take("home").unwrap();
        character.fiance = row.take("fiance").unwrap();
        character.partner = row.take("partner").unwrap();
        character.admin_level = AdminLevel::from(row.take::<i32, &str>("admin_level").unwrap());
        character.class = row.take("class").unwrap();
        character.gender = Gender::from(row.take::<i32, &str>("gender").unwrap());
        character.skin = row.take("race").unwrap();
        character.hair_style = row.take("hair_style").unwrap();
        character.hair_color = row.take("hair_color").unwrap();
        character.bank_level = row.take("bank_level").unwrap();
        character.gold_bank = row.take("gold_bank").unwrap();
        character.guild_rank = row.take("guild_rank").unwrap();
        character.guild_rank_string = row.take("guild_rank_string").unwrap();
        character.equipment.boots = row.take("boots").unwrap();
        character.equipment.accessory = row.take("accessory").unwrap();
        character.equipment.gloves = row.take("gloves").unwrap();
        character.equipment.belt = row.take("belt").unwrap();
        character.equipment.armor = row.take("armor").unwrap();
        character.equipment.hat = row.take("hat").unwrap();
        character.equipment.shield = row.take("shield").unwrap();
        character.equipment.weapon = row.take("weapon").unwrap();
        character.equipment.ring[0] = row.take("ring").unwrap();
        character.equipment.ring[1] = row.take("ring2").unwrap();
        character.equipment.armlet[0] = row.take("armlet").unwrap();
        character.equipment.armlet[1] = row.take("armlet2").unwrap();
        character.equipment.bracer[0] = row.take("bracer").unwrap();
        character.equipment.bracer[1] = row.take("bracer2").unwrap();
        character.equipment.necklace = row.take("necklace").unwrap();
        character.level = row.take("level").unwrap();
        character.experience = row.take("experience").unwrap();
        character.hp = row.take("hp").unwrap();
        character.tp = row.take("tp").unwrap();
        character.base_strength = row.take("strength").unwrap();
        character.base_intelligence = row.take("intelligence").unwrap();
        character.base_wisdom = row.take("wisdom").unwrap();
        character.base_agility = row.take("agility").unwrap();
        character.base_constitution = row.take("constitution").unwrap();
        character.base_charisma = row.take("charisma").unwrap();
        character.stat_points = row.take("stat_points").unwrap();
        character.skill_points = row.take("skill_points").unwrap();
        character.karma = row.take("karma").unwrap();
        character.usage = row.take("usage").unwrap();
        character.map_id = row.take("map").unwrap();
        character.coords.x = row.take("x").unwrap();
        character.coords.y = row.take("y").unwrap();
        character.direction = Direction::from(row.take::<i32, &str>("direction").unwrap());
        character.sit_state = SitState::from(row.take::<i32, &str>("sitting").unwrap());
        character.hidden = row.take::<u32, &str>("hidden").unwrap() == 1;
        character.guild_name = row.take("guild_name").unwrap();
        character.guild_tag = row.take("tag").unwrap();

        character.items = conn
            .exec_map(
                include_str!("../sql/get_character_inventory.sql"),
                params! {
                    "character_id" => id,
                },
                |mut row: Row| Item {
                    id: row.take(0).unwrap(),
                    amount: row.take(1).unwrap(),
                },
            )
            .await?;

        character.bank = conn
            .exec_map(
                include_str!("../sql/get_character_bank.sql"),
                params! {
                    "character_id" => id,
                },
                |mut row: Row| Item {
                    id: row.take(0).unwrap(),
                    amount: row.take(1).unwrap(),
                },
            )
            .await?;

        character.spells = conn
            .exec_map(
                include_str!("../sql/get_character_spells.sql"),
                params! {
                    "character_id" => id,
                },
                |mut row: Row| Spell {
                    id: row.take(0).unwrap(),
                    level: row.take(1).unwrap(),
                },
            )
            .await?;

        character.quests = conn
            .exec_map(
                include_str!("../sql/get_character_quest_progress.sql"),
                params! {
                    "character_id" => id,
                },
                |mut row: Row| QuestProgress {
                    id: row.take(0).unwrap(),
                    state: row.take(1).unwrap(),
                    npc_kills: {
                        let json = row.take::<String, usize>(2).unwrap();
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
                    player_kills: row.take(3).unwrap(),
                    done_at: row
                        .take::<Option<NaiveDateTime>, usize>(4)
                        .map(|done_at| {
                            done_at.map(|done_at| Utc.from_local_datetime(&done_at).unwrap())
                        })
                        .unwrap(),
                    completions: row.take(5).unwrap(),
                },
            )
            .await?;

        Ok(character)
    }
}
