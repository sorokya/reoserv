use eolib::protocol::{
    Coords,
    net::server::{
        LoginMessageCode, ServerSettings, WelcomeReplyServerPacketWelcomeCodeDataSelectCharacter,
    },
};

use crate::{
    CLASS_DB, ITEM_DB, NPC_DB, SETTINGS, SPELL_DB, character::Character, utils::pad_string,
};

use super::Player;

impl Player {
    pub async fn get_welcome_request_data(
        &mut self,
        character: &Character,
    ) -> Result<
        WelcomeReplyServerPacketWelcomeCodeDataSelectCharacter,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let (map_rid, map_file_size) = {
            let map = self.world.get_map(character.map_id).await?;
            map.get_rid_and_size().await?
        };

        let item_db = ITEM_DB.load();
        let class_db = CLASS_DB.load();
        let npc_db = NPC_DB.load();
        let spell_db = SPELL_DB.load();

        let (eif_rid, eif_length) = { (item_db.rid, item_db.items.len() as i32) };

        let (ecf_rid, ecf_length) = { (class_db.rid, class_db.classes.len() as i32) };

        let (enf_rid, enf_length) = { (npc_db.rid, npc_db.npcs.len() as i32) };

        let (esf_rid, esf_length) = { (spell_db.rid, spell_db.skills.len() as i32) };

        let settings = ServerSettings {
            jail_map: SETTINGS.load().jail.map,
            rescue_map: 4,
            rescue_coords: Coords { x: 24, y: 24 },
            spy_and_light_guide_flood_rate: 10,
            guardian_flood_rate: 10,
            game_master_flood_rate: 10,
            high_game_master_flood_rate: 0,
        };

        let session_id = self.generate_session_id();

        Ok(WelcomeReplyServerPacketWelcomeCodeDataSelectCharacter {
            session_id,
            character_id: character.id,
            map_id: character.map_id,
            map_rid,
            map_file_size,
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
            guild_tag: pad_string(&character.guild_tag.clone().unwrap_or_default(), 3),
            admin: character.admin_level,
            level: character.level,
            experience: character.experience,
            usage: character.usage,
            stats: character.get_character_stats_welcome(),
            equipment: character.get_equipment_welcome(),
            guild_rank: character.guild_rank.unwrap_or_default(),
            settings,
            login_message_code: match character.usage {
                0 => LoginMessageCode::Yes,
                _ => LoginMessageCode::No,
            },
        })
    }
}
