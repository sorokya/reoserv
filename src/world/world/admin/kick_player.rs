use super::super::World;

impl World {
    pub fn kick_player(&mut self, victim_name: String, admin_name: String, silent: bool) {
        let player_id = match self.characters.get(&victim_name) {
            Some(player_id) => player_id,
            None => return,
        };

        let player = match self.players.get(player_id) {
            Some(player) => player,
            None => return,
        };

        player.close("Player kicked".to_string());

        if !silent {
            self.broadcast_server_message(&format!(
                "Attention!! {} has been removed from the game -{} [jailed]",
                victim_name, admin_name
            ));
        }
    }
}
