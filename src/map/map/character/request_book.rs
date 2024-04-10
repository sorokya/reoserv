use eolib::protocol::net::{
    server::{BookReplyServerPacket, CharacterDetails},
    PacketAction, PacketFamily,
};

use crate::QUEST_DB;

use super::super::Map;

impl Map {
    pub fn request_book(&self, player_id: i32, target_player_id: i32) {
        let player = match self.characters.get(&player_id) {
            Some(character) => match &character.player {
                Some(player) => player.to_owned(),
                None => return,
            },
            None => return,
        };

        let target = match self.characters.get(&target_player_id) {
            Some(character) => character.to_owned(),
            None => return,
        };

        let world = self.world.clone();

        tokio::spawn(async move {
            let in_party = world.get_player_party(target_player_id).await.is_some();

            player.send(
                PacketAction::Reply,
                PacketFamily::Book,
                &BookReplyServerPacket {
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
                    quest_names: target
                        .quests
                        .iter()
                        .filter_map(|q| {
                            if q.done_at.is_some() && q.state != 0 {
                                QUEST_DB.get(&q.id).map(|quest| quest.name.to_owned())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>(),
                    icon: target.get_icon(in_party),
                },
            );
        });
    }
}
