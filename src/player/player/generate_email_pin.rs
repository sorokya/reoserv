use rand::Rng;

use super::super::Player;

impl Player {
    pub fn generate_email_pin(&mut self) -> String {
        let mut rng = rand::thread_rng();
        let pin: u32 = rng.gen_range(1000000..9999999);
        self.email_pin = Some(pin.to_string());
        pin.to_string()
    }
}
