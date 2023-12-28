use eolib::protocol::{AdminLevel, Gender, Direction, net::{server::SitState, Item, Spell}};
use mysql_async::{prelude::*, Conn, Params, Row};

use super::Character;

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
        character.guild_rank_id = row.take("guild_rank_id").unwrap();
        character.guild_rank_string = row.take("guild_rank_string").unwrap();
        character.paperdoll.boots = row.take("boots").unwrap();
        character.paperdoll.accessory = row.take("accessory").unwrap();
        character.paperdoll.gloves = row.take("gloves").unwrap();
        character.paperdoll.belt = row.take("belt").unwrap();
        character.paperdoll.armor = row.take("armor").unwrap();
        character.paperdoll.hat = row.take("hat").unwrap();
        character.paperdoll.shield = row.take("shield").unwrap();
        character.paperdoll.weapon = row.take("weapon").unwrap();
        character.paperdoll.ring[0] = row.take("ring").unwrap();
        character.paperdoll.ring[1] = row.take("ring2").unwrap();
        character.paperdoll.armlet[0] = row.take("armlet").unwrap();
        character.paperdoll.armlet[1] = row.take("armlet2").unwrap();
        character.paperdoll.bracer[0] = row.take("bracer").unwrap();
        character.paperdoll.bracer[1] = row.take("bracer2").unwrap();
        character.paperdoll.necklace = row.take("necklace").unwrap();
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

        Ok(character)
    }
}
