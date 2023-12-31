use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{CharacterDetails, PaperdollReplyServerPacket},
        PacketAction, PacketFamily,
    },
};

use super::super::Map;

impl Map {
    pub async fn request_paperdoll(&self, player_id: i32, target_player_id: i32) {
        let player = match self.characters.get(&player_id) {
            Some(character) => character.player.as_ref().unwrap(),
            None => {
                error!("Failed to get player");
                return;
            }
        };

        let target = match self.characters.get(&target_player_id) {
            Some(character) => character,
            None => {
                error!("Failed to get target");
                return;
            }
        };

        let in_party = self
            .world
            .get_player_party(target_player_id)
            .await
            .is_some();

        let reply = PaperdollReplyServerPacket {
            details: CharacterDetails {
                name: target.name.clone(),
                home: target.home.clone(),
                admin: target.admin_level,
                partner: match &target.partner {
                    Some(partner) => partner.clone(),
                    None => "".to_string(),
                },
                title: match &target.title {
                    Some(title) => title.clone(),
                    None => "".to_string(),
                },
                guild: match &target.guild_name {
                    Some(guild) => guild.clone(),
                    None => "".to_string(),
                },
                guild_rank: match &target.guild_rank_string {
                    Some(guild_rank) => guild_rank.clone(),
                    None => "".to_string(),
                },
                player_id: target_player_id,
                class_id: target.class,
                gender: target.gender,
            },
            equipment: target.paperdoll.clone(),
            icon: target.get_icon(in_party),
        };

        let mut writer = EoWriter::new();

        if let Err(e) = reply.serialize(&mut writer) {
            error!("Failed to serialize PaperdollReplyServerPacket: {}", e);
            return;
        }

        player.send(
            PacketAction::Reply,
            PacketFamily::Paperdoll,
            writer.to_byte_array(),
        );
    }
}
