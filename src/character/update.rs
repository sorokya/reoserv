use eo::{
    data::EOInt,
    protocol::{Item, Spell},
};
use mysql_async::{prelude::*, Conn, Row, TxOpts};

use super::Character;

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

        let mut tx = conn.start_transaction(TxOpts::default()).await?;

        tx.exec_drop(
            include_str!("../sql/update_character.sql"),
            params! {
                "character_id" => self.id,
                "title" => &self.title,
                "home" => &self.home,
                "fiance" => &self.fiance,
                "partner" => &self.partner,
                "admin_level" => self.admin_level as u32,
                "class" => self.class as u32,
                "gender" => self.gender as u32,
                "race" => self.skin as u32,
                "hair_style" => self.hair_style as u32,
                "hair_color" => self.hair_color as u32,
                "bank_max" => self.bank_max,
                "gold_bank" => self.gold_bank,
            },
        )
        .await?;

        tx.exec_drop(
            include_str!("../sql/update_paperdoll.sql"),
            params! {
                "character_id" => self.id,
                "boots" => self.paperdoll.boots as u32,
                "accessory" => self.paperdoll.accessory as u32,
                "gloves" => self.paperdoll.gloves as u32,
                "belt" => self.paperdoll.belt as u32,
                "armor" => self.paperdoll.armor as u32,
                "necklace" => self.paperdoll.necklace as u32,
                "hat" => self.paperdoll.hat as u32,
                "shield" => self.paperdoll.shield as u32,
                "weapon" => self.paperdoll.weapon as u32,
                "ring" => self.paperdoll.ring[0] as u32,
                "ring2" => self.paperdoll.ring[1] as u32,
                "armlet" => self.paperdoll.armlet[0] as u32,
                "armlet2" => self.paperdoll.armlet[1] as u32,
                "bracer" => self.paperdoll.bracer[0] as u32,
                "bracer2" => self.paperdoll.bracer[1] as u32,
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
                "direction" => self.direction as u32,
                "sitting" => self.sit_state as u32,
                "hidden" => EOInt::from(self.hidden),
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

        // TODO: save bank

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
            if !old_spells.contains(spell) {
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
            if !self.items.contains(item) {
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
            if !old_items.contains(item) {
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
            if !self.bank.contains(item) {
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
            if !old_bank.contains(item) {
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

        tx.commit().await?;

        Ok(())
    }
}
