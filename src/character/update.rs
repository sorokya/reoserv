use eolib::protocol::net::{Item, Spell};
use mysql_async::{prelude::*, Conn, Row, TxOpts};

use super::{Character, QuestProgress};

impl Character {
    pub async fn update(
        &self,
        conn: &mut Conn,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let old_items = conn
            .exec_map(
                include_str!("../sql/get_character_inventory.sql"),
                params! {
                    "character_id" => self.id,
                },
                |mut row: Row| Item {
                    id: row.take(0).unwrap(),
                    amount: row.take(1).unwrap(),
                },
            )
            .await?;

        let old_bank = conn
            .exec_map(
                include_str!("../sql/get_character_bank.sql"),
                params! {
                    "character_id" => self.id,
                },
                |mut row: Row| Item {
                    id: row.take(0).unwrap(),
                    amount: row.take(1).unwrap(),
                },
            )
            .await?;

        let old_spells = conn
            .exec_map(
                include_str!("../sql/get_character_spells.sql"),
                params! {
                    "character_id" => self.id,
                },
                |mut row: Row| Spell {
                    id: row.take(0).unwrap(),
                    level: row.take(1).unwrap(),
                },
            )
            .await?;

        let old_quests = conn
            .exec_map(
                include_str!("../sql/get_character_quest_progress.sql"),
                params! {
                    "character_id" => self.id,
                },
                |mut row: Row| QuestProgress {
                    id: row.take(0).unwrap(),
                    ..Default::default()
                },
            )
            .await?;

        let mut tx = conn.start_transaction(TxOpts::default()).await?;

        tx.exec_drop(
            include_str!("../sql/update_character.sql"),
            params! {
                "character_id" => self.id,
                "title" => &self.title,
                "home" => &self.home,
                "fiance" => &self.fiance,
                "partner" => &self.partner,
                "admin_level" => i32::from(self.admin_level),
                "class" => self.class as u32,
                "gender" => i32::from(self.gender),
                "race" => self.skin,
                "hair_style" => self.hair_style as u32,
                "hair_color" => self.hair_color as u32,
                "bank_level" => self.bank_level,
                "gold_bank" => self.gold_bank,
                "guild_tag" => &self.guild_tag,
                "guild_rank" => self.guild_rank,
                "guild_rank_string" => &self.guild_rank_string,
            },
        )
        .await?;

        tx.exec_drop(
            include_str!("../sql/update_paperdoll.sql"),
            params! {
                "character_id" => self.id,
                "boots" => self.equipment.boots as u32,
                "accessory" => self.equipment.accessory as u32,
                "gloves" => self.equipment.gloves as u32,
                "belt" => self.equipment.belt as u32,
                "armor" => self.equipment.armor as u32,
                "necklace" => self.equipment.necklace as u32,
                "hat" => self.equipment.hat as u32,
                "shield" => self.equipment.shield as u32,
                "weapon" => self.equipment.weapon as u32,
                "ring" => self.equipment.ring[0] as u32,
                "ring2" => self.equipment.ring[1] as u32,
                "armlet" => self.equipment.armlet[0] as u32,
                "armlet2" => self.equipment.armlet[1] as u32,
                "bracer" => self.equipment.bracer[0] as u32,
                "bracer2" => self.equipment.bracer[1] as u32,
            },
        )
        .await?;

        tx.exec_drop(
            include_str!("../sql/update_position.sql"),
            params! {
                "character_id" => self.id,
                "map_id" => self.map_id as u32,
                "x" => self.coords.x as u32,
                "y" => self.coords.y as u32,
                "direction" => i32::from(self.direction),
                "sitting" => i32::from(self.sit_state),
                "hidden" => i32::from(self.hidden),
            },
        )
        .await?;

        tx.exec_drop(
            include_str!("../sql/update_stats.sql"),
            params! {
                "character_id" => self.id,
                "level" => self.level as u32,
                "experience" => self.experience,
                "hp" => self.hp as u32,
                "tp" => self.tp as u32,
                "strength" => self.base_strength as u32,
                "intelligence" => self.base_intelligence as u32,
                "wisdom" => self.base_wisdom as u32,
                "agility" => self.base_agility as u32,
                "constitution" => self.base_constitution as u32,
                "charisma" => self.base_charisma as u32,
                "stat_points" => self.stat_points as u32,
                "skill_points" => self.skill_points as u32,
                "karma" => self.karma as u32,
                "usage" => self.usage,
            },
        )
        .await?;

        for spell in &old_spells {
            if !self.has_spell(spell.id) {
                tx.exec_drop(
                    include_str!("../sql/delete_spell.sql"),
                    params! {
                        "character_id" => self.id,
                        "spell_id" => spell.id,
                    },
                )
                .await?;
            }
        }

        for spell in &self.spells {
            if !old_spells.iter().any(|s| s.id == spell.id) {
                tx.exec_drop(
                    include_str!("../sql/create_spell.sql"),
                    params! {
                        "character_id" => self.id,
                        "spell_id" => spell.id,
                        "level" => spell.level,
                    },
                )
                .await?;
            } else {
                tx.exec_drop(
                    include_str!("../sql/update_spell.sql"),
                    params! {
                        "character_id" => self.id,
                        "spell_id" => spell.id,
                        "level" => spell.level,
                    },
                )
                .await?;
            }
        }

        for item in &old_items {
            if !self.items.iter().any(|i| i.id == item.id) {
                tx.exec_drop(
                    include_str!("../sql/delete_inventory_item.sql"),
                    params! {
                        "character_id" => self.id,
                        "item_id" => item.id,
                    },
                )
                .await?;
            }
        }

        for item in &self.items {
            if !old_items.iter().any(|i| i.id == item.id) {
                tx.exec_drop(
                    include_str!("../sql/create_inventory_item.sql"),
                    params! {
                        "character_id" => self.id,
                        "item_id" => item.id,
                        "quantity" => item.amount,
                    },
                )
                .await?;
            } else {
                tx.exec_drop(
                    include_str!("../sql/update_inventory_item.sql"),
                    params! {
                        "character_id" => self.id,
                        "item_id" => item.id,
                        "quantity" => item.amount,
                    },
                )
                .await?;
            }
        }

        for item in &old_bank {
            if !self.bank.iter().any(|i| i.id == item.id) {
                tx.exec_drop(
                    include_str!("../sql/delete_bank_item.sql"),
                    params! {
                        "character_id" => self.id,
                        "item_id" => item.id,
                    },
                )
                .await?;
            }
        }

        for item in &self.bank {
            if !old_bank.iter().any(|i| i.id == item.id) {
                tx.exec_drop(
                    include_str!("../sql/create_bank_item.sql"),
                    params! {
                        "character_id" => self.id,
                        "item_id" => item.id,
                        "quantity" => item.amount,
                    },
                )
                .await?;
            } else {
                tx.exec_drop(
                    include_str!("../sql/update_bank_item.sql"),
                    params! {
                        "character_id" => self.id,
                        "item_id" => item.id,
                        "quantity" => item.amount,
                    },
                )
                .await?;
            }
        }

        for quest in &old_quests {
            if !self.quests.iter().any(|q| q.id == quest.id) {
                tx.exec_drop(
                    include_str!("../sql/delete_quest_progress.sql"),
                    params! {
                        "character_id" => self.id,
                        "quest_id" => quest.id,
                    },
                )
                .await?;
            }
        }

        for quest in &self.quests {
            let mut npc_kills = String::from('{');
            for (index, (npc_id, kills)) in quest.npc_kills.iter().enumerate() {
                npc_kills.push_str(&format!(
                    "\"{}\":{}{}",
                    npc_id,
                    kills,
                    if index < quest.npc_kills.len() - 1 {
                        ","
                    } else {
                        ""
                    }
                ));
            }
            npc_kills.push('}');

            if !old_quests.iter().any(|q| q.id == quest.id) {
                tx.exec_drop(
                    include_str!("../sql/create_quest_progress.sql"),
                    params! {
                        "character_id" => self.id,
                        "quest_id" => quest.id,
                        "state" => quest.state,
                        "npc_kills" => npc_kills,
                        "player_kills" => quest.player_kills,
                    },
                )
                .await?;
            } else {
                tx.exec_drop(
                    include_str!("../sql/update_quest_progress.sql"),
                    params! {
                        "character_id" => self.id,
                        "quest_id" => quest.id,
                        "state" => quest.state,
                        "npc_kills" => npc_kills,
                        "player_kills" => quest.player_kills,
                        "done" => quest.done,
                    },
                )
                .await?;
            }
        }

        tx.commit().await?;

        Ok(())
    }
}
