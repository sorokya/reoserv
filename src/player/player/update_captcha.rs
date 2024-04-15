use eolib::protocol::net::{PacketAction, PacketFamily};
use rand::Rng;

use crate::deep::{CaptchaAgreeServerPacket, FAMILY_CAPTCHA};

use super::super::Player;

impl Player {
    pub async fn update_captcha(&mut self) {
        let captcha = match &mut self.captcha {
            Some(captcha) => captcha,
            None => return,
        };

        captcha.challenge = {
            let mut rng = rand::thread_rng();
            (0..5)
                .map(|_| {
                    rng.gen_range(65..=90) as u8 as char // ASCII codes for upper case letters
                })
                .collect()
        };

        captcha.attempts = 0;

        let _ = self
            .bus
            .send(
                PacketAction::Agree,
                PacketFamily::Unrecognized(FAMILY_CAPTCHA),
                CaptchaAgreeServerPacket {
                    id: 1,
                    captcha: captcha.challenge.clone(),
                },
            )
            .await;
    }
}
