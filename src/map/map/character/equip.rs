use eolib::protocol::{
    net::{
        server::{
            AvatarAgreeServerPacket, AvatarChange, AvatarChangeChangeTypeData,
            AvatarChangeChangeTypeDataEquipment, AvatarChangeType, PaperdollAgreeServerPacket,
        },
        PacketAction, PacketFamily,
    },
    r#pub::ItemType,
};

use crate::{
    character::EquipResult,
    deep::{PaperdollSwapServerPacket, ACTION_SWAP},
    ITEM_DB,
};

use super::super::Map;

impl Map {
    pub fn equip(&mut self, player_id: i32, item_id: i32, sub_loc: i32) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        if !character.items.iter().any(|i| i.id == item_id) {
            return;
        }

        let result = character.equip(item_id, sub_loc);

        if result == EquipResult::Failed {
            return;
        }

        let change = AvatarChange {
            player_id,
            change_type: AvatarChangeType::Equipment,
            sound: false,
            change_type_data: Some(AvatarChangeChangeTypeData::Equipment(
                AvatarChangeChangeTypeDataEquipment {
                    equipment: character.get_equipment_change(),
                },
            )),
        };

        if let Some(player) = character.player.as_ref() {
            if let EquipResult::Swapped(removed_item_id) = result {
                player.send(
                    PacketAction::Unrecognized(ACTION_SWAP),
                    PacketFamily::Paperdoll,
                    &PaperdollSwapServerPacket {
                        change: change.clone(),
                        item_id,
                        remaining_amount: character.get_item_amount(item_id),
                        removed_item_id,
                        removed_item_amount: character.get_item_amount(removed_item_id),
                        stats: character.get_character_stats_equipment_change(),
                    },
                );
            } else {
                player.send(
                    PacketAction::Agree,
                    PacketFamily::Paperdoll,
                    &PaperdollAgreeServerPacket {
                        change: change.clone(),
                        item_id,
                        remaining_amount: character.get_item_amount(item_id),
                        sub_loc,
                        stats: character.get_character_stats_equipment_change(),
                    },
                );
            }
        }

        if character.hidden {
            return;
        }

        let is_visible_change = matches!(
            ITEM_DB.items.get(item_id as usize - 1).unwrap().r#type,
            ItemType::Armor | ItemType::Weapon | ItemType::Shield | ItemType::Hat | ItemType::Boots
        );

        if is_visible_change {
            self.send_packet_near_player(
                player_id,
                PacketAction::Agree,
                PacketFamily::Avatar,
                &AvatarAgreeServerPacket { change },
            );
        }
    }
}
