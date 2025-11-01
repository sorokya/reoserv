use crate::LANG;

use super::super::World;

impl World {
    pub fn toggle_global(&mut self, admin_name: String) {
        if self.global_locked {
            self.global_locked = false;
            self.broadcast_server_message(&get_lang_string!(
                &LANG.announce_global,
                name = admin_name,
                state = "on"
            ));
        } else {
            self.global_locked = true;
            self.broadcast_server_message(&get_lang_string!(
                &LANG.announce_global,
                name = admin_name,
                state = "off"
            ));
        }
    }
}
