use std::convert::TryInto;

use eo::protocol::{server::welcome::ReplySelectCharacter, ServerSettings};

use crate::{
    character::Character, errors::DataNotFoundError, player::PlayerHandle, CLASS_DB, ITEM_DB,
    NPC_DB, SETTINGS, SPELL_DB,
};

use super::World;

impl World {
    pub async fn get_welcome_request_data(
        &self,
        player: PlayerHandle,
        character: &Character,
    ) -> Result<ReplySelectCharacter, Box<dyn std::error::Error + Send + Sync>> {
        let (map_rid, map_filesize) = {
            let maps = self.maps.as_ref().expect("Maps not loaded");
            let map = match maps.get(&character.map_id) {
                Some(map) => map,
                None => {
                    error!("Map not found: {}", character.map_id);
                    return Err(Box::new(DataNotFoundError::new(
                        "Map".to_string(),
                        character.map_id,
                    )));
                }
            };
            map.get_rid_and_size().await
        };

        let (eif_rid, eif_length) = { (ITEM_DB.rid, ITEM_DB.num_items) };

        let (ecf_rid, ecf_length) = { (CLASS_DB.rid, CLASS_DB.num_classes) };

        let (enf_rid, enf_length) = { (NPC_DB.rid, NPC_DB.num_npcs) };

        let (esf_rid, esf_length) = { (SPELL_DB.rid, SPELL_DB.num_spells) };

        let settings = ServerSettings {
            jail_map: SETTINGS.jail.map.try_into().expect("Invalid map id"),
            rescue_map: 4,
            rescue_x: 24,
            rescue_y: 24,
            light_guide_flood_rate: 10,
            guardian_flood_rate: 10,
            gm_flood_rate: 10,
            hgm_flood_rate: 0,
        };

        let session_id = player.generate_session_id().await;

        Ok(ReplySelectCharacter {
            session_id,
            character_id: character.id,
            map_id: character.map_id,
            map_rid,
            map_filesize,
            eif_rid,
            eif_length,
            enf_rid,
            enf_length,
            esf_rid,
            esf_length,
            ecf_rid,
            ecf_length,
            name: character.name.to_string(),
            title: character.title.clone().unwrap_or_default(),
            guild_name: character.guild_name.clone().unwrap_or_default(),
            guild_rank_name: character.guild_rank_string.clone().unwrap_or_default(),
            class_id: character.class,
            guild_tag: character.guild_tag.clone().unwrap_or_default(),
            admin: character.admin_level,
            level: character.level,
            experience: character.experience,
            usage: character.usage,
            stats: character.get_character_stats_2(),
            paperdoll: character.paperdoll.to_owned(),
            guild_rank: character.guild_rank_id.unwrap_or_default(),
            settings,
            login_message: match character.usage {
                0 => 2,
                _ => 0,
            },
        })
    }
}
