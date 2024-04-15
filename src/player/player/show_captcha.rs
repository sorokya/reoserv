use eolib::protocol::net::{PacketAction, PacketFamily};
use rand::Rng;

use crate::{
    deep::{CaptchaOpenServerPacket, FAMILY_CAPTCHA},
    player::Captcha,
    utils::is_deep,
};

use super::super::Player;

impl Player {
    pub async fn show_captcha(&mut self, experience: i32) {
        if !is_deep(&self.version) {
            return;
        }

        let captcha: String = {
            let mut rng = rand::thread_rng();
            (0..5)
                .map(|_| {
                    rng.gen_range(65..=90) as u8 as char // ASCII codes for upper case letters
                })
                .collect()
        };

        self.captcha = Some(Captcha {
            challenge: captcha.to_owned(),
            reward: experience,
            attempts: 0,
        });

        if let Some(map) = &self.map {
            map.open_captcha(self.id);
        }

        let _ = self
            .bus
            .send(
                PacketAction::Open,
                PacketFamily::Unrecognized(FAMILY_CAPTCHA),
                CaptchaOpenServerPacket {
                    id: 1,
                    reward_exp: experience,
                    captcha: Some(captcha),
                },
            )
            .await;
    }
}
