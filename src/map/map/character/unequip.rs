use eolib::protocol::{
    net::{
        server::{
            AvatarAgreeServerPacket, AvatarChange, AvatarChangeChangeTypeData,
            AvatarChangeChangeTypeDataEquipment, AvatarChangeType, PaperdollRemoveServerPacket,
        },
        PacketAction, PacketFamily,
    },
    r#pub::ItemType,
};

use crate::ITEM_DB;

use super::super::Map;

impl Map {
    pub fn unequip(&mut self, player_id: i32, item_id: i32, sub_loc: i32) {
        let target = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => {
                error!("Failed to get character");
                return;
            }
        };

        if !target.unequip(item_id, sub_loc) {
            return;
        }

        let change = AvatarChange {
            player_id,
            change_type: AvatarChangeType::Equipment,
            sound: false,
            change_type_data: Some(AvatarChangeChangeTypeData::Equipment(
                AvatarChangeChangeTypeDataEquipment {
                    equipment: target.get_equipment_change(),
                },
            )),
        };

        target.player.as_ref().unwrap().send(
            PacketAction::Remove,
            PacketFamily::Paperdoll,
            &PaperdollRemoveServerPacket {
                change: change.clone(),
                item_id,
                sub_loc,
                stats: target.get_character_stats_equipment_change(),
            },
        );

        if target.hidden {
            return;
        }

        let is_visible_change = matches!(
            ITEM_DB.items.get(item_id as usize - 1).unwrap().r#type,
            ItemType::Armor | ItemType::Weapon | ItemType::Shield | ItemType::Hat | ItemType::Boots
        );

        if is_visible_change && self.characters.len() > 1 {
            self.send_packet_near_player(
                player_id,
                PacketAction::Agree,
                PacketFamily::Avatar,
                &AvatarAgreeServerPacket { change },
            );
        }
    }
}
