use eolib::{
    data::{EoReader, EoSerialize},
    protocol::{
        net::{
            client::{
                TalkAdminClientPacket, TalkAnnounceClientPacket, TalkMsgClientPacket,
                TalkOpenClientPacket, TalkReportClientPacket, TalkRequestClientPacket,
                TalkTellClientPacket,
            },
            PacketAction,
        },
        AdminLevel,
    },
};

use super::{
    super::Player,
    handle_command::handle_command,
    handle_player_command::{handle_player_command, PlayerCommandResult},
};

impl Player {
    fn talk_admin(&mut self, reader: EoReader) {
        let admin = match TalkAdminClientPacket::deserialize(&reader) {
            Ok(admin) => admin,
            Err(e) => {
                error!("Error deserializing TalkAdminClientPacket {}", e);
                return;
            }
        };

        let map = match &self.map {
            Some(map) => map.to_owned(),
            None => return,
        };

        let world = self.world.clone();

        let player_id = self.id;

        tokio::spawn(async move {
            if let Ok(Some(character)) = map.get_character(player_id).await {
                if i32::from(character.admin_level) >= i32::from(AdminLevel::Guardian) {
                    world.broadcast_admin_message(character.name, admin.message);
                }
            }
        });
    }

    fn talk_announce(&mut self, reader: EoReader) {
        let announce = match TalkAnnounceClientPacket::deserialize(&reader) {
            Ok(announce) => announce,
            Err(e) => {
                error!("Error deserializing TalkAnnounceClientPacket {}", e);
                return;
            }
        };

        let map = match &self.map {
            Some(map) => map.to_owned(),
            None => return,
        };

        let world = self.world.clone();

        let player_id = self.id;

        tokio::spawn(async move {
            if let Ok(Some(character)) = map.get_character(player_id).await {
                if i32::from(character.admin_level) >= i32::from(AdminLevel::Guardian) {
                    world.broadcast_announcement(character.name, announce.message);
                }
            }
        });
    }

    fn talk_msg(&mut self, reader: EoReader) {
        let msg = match TalkMsgClientPacket::deserialize(&reader) {
            Ok(msg) => msg,
            Err(e) => {
                error!("Error deserializing TalkMsgClientPacket {}", e);
                return;
            }
        };

        let map = match &self.map {
            Some(map) => map.to_owned(),
            None => return,
        };

        let world = self.world.clone();

        let player_id = self.id;

        tokio::spawn(async move {
            if let Ok(Some(character)) = map.get_character(player_id).await {
                world.broadcast_global_message(
                    character.player_id.unwrap(),
                    character.name,
                    msg.message,
                )
            }
        });
    }

    fn talk_report(&mut self, reader: EoReader) {
        let report = match TalkReportClientPacket::deserialize(&reader) {
            Ok(report) => report,
            Err(e) => {
                error!("Error deserializing TalkReportClientPacket {}", e);
                return;
            }
        };

        let map = match &self.map {
            Some(map) => map.to_owned(),
            None => return,
        };

        let world = self.world.to_owned();

        let player_id = self.id;

        tokio::spawn(async move {
            let character = match map.get_character(player_id).await.expect("Failed to get character. Timeout") {
                Some(character) => character,
                None => return,
            };

            if report.message.starts_with('$') && character.admin_level != AdminLevel::Player {
                let args: Vec<&str> = report.message[1..].split_whitespace().collect();
                if !args.is_empty() {
                    let player = match world.get_player(player_id).await.expect("Failed to get player. Timeout") {
                        Some(player) => player,
                        None => return,
                    };

                    handle_command(args.as_slice(), &character, player, world).await;
                    return;
                }
            }

            if report.message.starts_with('#') {
                let args: Vec<&str> = report.message[1..].split_whitespace().collect();
                if !args.is_empty() {
                    let player = match world.get_player(player_id).await.expect("Failed to get player. Timeout") {
                        Some(player) => player,
                        None => return,
                    };

                    if handle_player_command(player_id, args.as_slice(), &player, &map).await
                        == PlayerCommandResult::NotFound
                    {
                        map.send_chat_message(player_id, report.message);
                    }
                    return;
                }
            }

            map.send_chat_message(player_id, report.message);
        });
    }

    fn talk_tell(&mut self, reader: EoReader) {
        let tell = match TalkTellClientPacket::deserialize(&reader) {
            Ok(tell) => tell,
            Err(e) => {
                error!("Error deserializing TalkTellClientPacket {}", e);
                return;
            }
        };

        self.world
            .send_private_message(self.id, tell.name, tell.message);
    }

    fn talk_open(&mut self, reader: EoReader) {
        let open = match TalkOpenClientPacket::deserialize(&reader) {
            Ok(open) => open,
            Err(e) => {
                error!("Error deserializing TalkOpenClientPacket {}", e);
                return;
            }
        };

        self.world.broadcast_party_message(self.id, open.message);
    }

    fn talk_request(&mut self, reader: EoReader) {
        let request = match TalkRequestClientPacket::deserialize(&reader) {
            Ok(request) => request,
            Err(e) => {
                error!("Error deserializing TalkRequestClientPacket {}", e);
                return;
            }
        };

        let map = match &self.map {
            Some(map) => map.to_owned(),
            None => return,
        };

        let world = self.world.to_owned();

        let player_id = self.id;

        tokio::spawn(async move {
            let character = match map.get_character(player_id).await.expect("Failed to get character. Timeout") {
                Some(character) => character,
                None => return,
            };

            let guild_tag = match character.guild_tag {
                Some(guild_tag) => guild_tag,
                None => return,
            };

            world.broadcast_guild_message(
                character.player_id,
                guild_tag,
                character.name,
                request.message,
            );
        });
    }

    pub fn handle_talk(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Admin => self.talk_admin(reader),
            PacketAction::Announce => self.talk_announce(reader),
            PacketAction::Msg => self.talk_msg(reader),
            PacketAction::Report => self.talk_report(reader),
            PacketAction::Tell => self.talk_tell(reader),
            PacketAction::Open => self.talk_open(reader),
            PacketAction::Request => self.talk_request(reader),
            _ => error!("Unhandled packet Talk_{:?}", action),
        }
    }
}
