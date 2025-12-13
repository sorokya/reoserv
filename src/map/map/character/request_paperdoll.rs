use eolib::protocol::net::{
    server::{CharacterDetails, PaperdollReplyServerPacket},
    PacketAction, PacketFamily,
};

use super::super::Map;

impl Map {
    pub fn request_paperdoll(&self, player_id: i32, target_player_id: i32) {
        let player = match self.characters.get(&player_id) {
            Some(character) => match character.player.as_ref() {
                Some(player) => player.to_owned(),
                None => return,
            },
            None => return,
        };

        let target = match self.characters.get(&target_player_id) {
            Some(character) => character.to_owned(),
            None => {
                error!("Failed to get target");
                return;
            }
        };

        let world = self.world.to_owned();

        tokio::spawn(async move {
            let in_party = world.get_player_party(target_player_id).await.expect("Failed to get player party. Timeout").is_some();

            player.send(
                PacketAction::Reply,
                PacketFamily::Paperdoll,
                &PaperdollReplyServerPacket {
                    details: CharacterDetails {
                        name: target.name.to_owned(),
                        home: target.home.to_owned(),
                        admin: target.admin_level,
                        partner: match &target.partner {
                            Some(partner) => partner.to_owned(),
                            None => "".to_string(),
                        },
                        title: match &target.title {
                            Some(title) => title.to_owned(),
                            None => "".to_string(),
                        },
                        guild: match &target.guild_name {
                            Some(guild) => guild.to_owned(),
                            None => "".to_string(),
                        },
                        guild_rank: match &target.guild_rank_string {
                            Some(guild_rank) => guild_rank.to_owned(),
                            None => "".to_string(),
                        },
                        player_id: target_player_id,
                        class_id: target.class,
                        gender: target.gender,
                    },
                    equipment: target.equipment.to_owned(),
                    icon: target.get_icon(in_party),
                },
            );
        });
    }
}
