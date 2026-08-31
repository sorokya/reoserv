use super::super::Player;

impl Player {
    pub fn generate_email_pin(&mut self) -> String {
        let pin = self.rng.rand_range(1000000..9999998);
        self.email_pin = Some(pin.to_string());
        pin.to_string()
    }
}
