use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::PacketAction,
};

use crate::{
    deep::{CaptchaReplyClientPacket, CaptchaRequestClientPacket},
    utils::is_deep,
};

use super::super::Player;

impl Player {
    async fn captcha_reply(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let reply = match CaptchaReplyClientPacket::deserialize(&reader) {
                Ok(reply) => reply,
                Err(e) => {
                    error!("Failed to deserialize CaptchaReplyClientPacket: {}", e);
                    return;
                }
            };

            let (attempts, captcha, reward) = match &self.captcha {
                Some(captcha) => (
                    captcha.attempts + 1,
                    captcha.challenge.to_owned(),
                    captcha.reward,
                ),
                None => return,
            };

            if attempts > 5 {
                return;
            }

            if reply.captcha != captcha {
                if let Some(captcha) = &mut self.captcha {
                    captcha.attempts += 1;
                }
                return;
            }

            self.captcha = None;

            map.close_captcha(self.id, reward);
        }
    }

    async fn captcha_request(&mut self, reader: EoReader) {
        if let Err(e) = CaptchaRequestClientPacket::deserialize(&reader) {
            error!("Failed to deserialize CaptchaRequestClientPacket: {}", e);
            return;
        }

        if self.captcha.is_none() {
            return;
        }

        self.update_captcha().await;
    }

    pub async fn handle_captcha(&mut self, action: PacketAction, reader: EoReader) {
        if !is_deep(&self.version) {
            return;
        }

        match action {
            PacketAction::Reply => self.captcha_reply(reader).await,
            PacketAction::Request => self.captcha_request(reader).await,
            _ => error!("Unhandled packet Captcha_{:?}", action),
        }
    }
}
