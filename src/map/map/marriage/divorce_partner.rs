use eolib::protocol::net::{
    PacketAction, PacketFamily,
    server::{MarriageReply, MarriageReplyServerPacket},
};

use super::super::Map;

impl Map {
    pub fn divorce_partner(&mut self, player_id: i32) {
        if let Some(character) = self.characters.get_mut(&player_id) {
            character.partner = None;
            if let Some(player) = character.player.as_ref() {
                player.send(
                    PacketAction::Reply,
                    PacketFamily::Marriage,
                    &MarriageReplyServerPacket {
                        reply_code: MarriageReply::DivorceNotification,
                        reply_code_data: None,
                    },
                );
            }
        }
    }
}
