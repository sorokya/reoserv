use eo::{
    data::{EOShort, Serializeable, StreamBuilder},
    protocol::{server::paperdoll, PacketAction, PacketFamily, PaperdollInfo},
};

use super::Map;

impl Map {
    pub async fn request_paperdoll(&self, player_id: EOShort, target_player_id: EOShort) {
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

        let reply = paperdoll::Reply {
            info: PaperdollInfo {
                name: target.name.clone(),
                home: target.home.clone(),
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
            paperdoll: target.paperdoll.clone(),
            paperdoll_icon: target.get_icon(in_party),
        };

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);

        player.send(PacketAction::Reply, PacketFamily::Paperdoll, builder.get());
    }
}
