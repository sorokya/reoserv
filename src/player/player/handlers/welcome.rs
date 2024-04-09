use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{
            WelcomeAgreeClientPacket, WelcomeAgreeClientPacketFileTypeData, WelcomeMsgClientPacket,
            WelcomeRequestClientPacket,
        },
        PacketAction,
    },
};



use super::super::Player;

impl Player {
    async fn welcome_agree(&mut self, reader: EoReader) {
        let agree = match WelcomeAgreeClientPacket::deserialize(&reader) {
            Ok(agree) => agree,
            Err(e) => {
                error!("Error deserializing WelcomeAgreeClientPacket {}", e);
                return;
            }
        };

        self.get_file(
            agree.file_type,
            agree.session_id,
            match agree.file_type_data {
                Some(WelcomeAgreeClientPacketFileTypeData::Emf(_)) => None,
                Some(WelcomeAgreeClientPacketFileTypeData::Eif(agree_item)) => {
                    Some(agree_item.file_id)
                }
                Some(WelcomeAgreeClientPacketFileTypeData::Enf(agree_npc)) => {
                    Some(agree_npc.file_id)
                }
                Some(WelcomeAgreeClientPacketFileTypeData::Esf(agree_spell)) => {
                    Some(agree_spell.file_id)
                }
                Some(WelcomeAgreeClientPacketFileTypeData::Ecf(agree_class)) => {
                    Some(agree_class.file_id)
                }
                _ => return,
            },
            false,
        )
        .await;
    }

    async fn welcome_msg(&mut self, reader: EoReader) {
        let msg = match WelcomeMsgClientPacket::deserialize(&reader) {
            Ok(msg) => msg,
            Err(e) => {
                error!("Error deserializing WelcomeMsgClientPacket {}", e);
                return;
            }
        };

        self.enter_game(msg.session_id).await;
    }

    async fn welcome_request(&mut self, reader: EoReader) {
        let request = match WelcomeRequestClientPacket::deserialize(&reader) {
            Ok(request) => request,
            Err(e) => {
                error!("Error deserializing WelcomeRequestClientPacket {}", e);
                return;
            }
        };

        self.select_character(request.character_id).await;
    }

    pub async fn handle_welcome(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Agree => self.welcome_agree(reader).await,
            PacketAction::Msg => self.welcome_msg(reader).await,
            PacketAction::Request => self.welcome_request(reader).await,
            _ => error!("Unhandled packet Welcome_{:?}", action),
        }
    }
}
