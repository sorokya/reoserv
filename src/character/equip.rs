use eolib::protocol::{
    net::{server::PaperdollPingServerPacket, PacketAction, PacketFamily},
    r#pub::ItemType,
};

use crate::ITEM_DB;

use super::{Character, EquipResult};

impl Character {
    pub fn equip(&mut self, item_id: i32, sub_loc: i32) -> EquipResult {
        if sub_loc > 1 {
            return EquipResult::Failed;
        }

        let item_record = match ITEM_DB.items.get(item_id as usize - 1) {
            Some(item) => item,
            None => return EquipResult::Failed,
        };

        if item_record.r#type == ItemType::Armor && item_record.spec2 != i32::from(self.gender) {
            return EquipResult::Failed;
        }

        if self.level < item_record.level_requirement
            || self.adj_strength < item_record.str_requirement
            || self.adj_intelligence < item_record.int_requirement
            || self.adj_wisdom < item_record.wis_requirement
            || self.adj_agility < item_record.agi_requirement
            || self.adj_constitution < item_record.con_requirement
            || self.adj_charisma < item_record.cha_requirement
        {
            return EquipResult::Failed;
        }

        if item_record.class_requirement != 0 && item_record.class_requirement != self.class {
            if let Some(player) = self.player.as_ref() {
                player.send(
                    PacketAction::Ping,
                    PacketFamily::Paperdoll,
                    &PaperdollPingServerPacket {
                        class_id: self.class,
                    },
                );
            }

            return EquipResult::Failed;
        }

        let mut result = EquipResult::Equiped;

        {
            let equipment_slot = match item_record.r#type {
                ItemType::Weapon => &mut self.equipment.weapon,
                ItemType::Shield => &mut self.equipment.shield,
                ItemType::Armor => &mut self.equipment.armor,
                ItemType::Hat => &mut self.equipment.hat,
                ItemType::Boots => &mut self.equipment.boots,
                ItemType::Gloves => &mut self.equipment.gloves,
                ItemType::Accessory => &mut self.equipment.accessory,
                ItemType::Belt => &mut self.equipment.belt,
                ItemType::Necklace => &mut self.equipment.necklace,
                ItemType::Ring => &mut self.equipment.ring[sub_loc as usize],
                ItemType::Armlet => &mut self.equipment.armlet[sub_loc as usize],
                ItemType::Bracer => &mut self.equipment.bracer[sub_loc as usize],
                _ => {
                    warn!(
                        "{} tried to equip an invalid item type: {:?}",
                        self.name, item_record.r#type
                    );
                    return EquipResult::Failed;
                }
            };

            if *equipment_slot != 0 {
                if self.is_deep {
                    result = EquipResult::Swapped(*equipment_slot);
                    *equipment_slot = item_id;
                } else {
                    return EquipResult::Failed;
                }
            } else {
                *equipment_slot = item_id;
            }
        }

        if let EquipResult::Swapped(item_id) = result {
            self.add_item_no_quest_rules(item_id, 1);
        }

        self.remove_item_no_quest_rules(item_id, 1);

        self.calculate_stats();

        result
    }
}
