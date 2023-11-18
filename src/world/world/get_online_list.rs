use eo::protocol::OnlinePlayers;

use super::World;

impl World {
    pub async fn get_online_list(&self) -> Vec<OnlinePlayers> {
        let mut online_list = Vec::new();
        for (player_id, player) in self.players.iter() {
            if let Ok(character) = player.get_character().await {
                if character.hidden {
                    continue;
                }

                let mut entry = OnlinePlayers::new();
                entry.name = character.name.to_string();
                entry.class_id = character.class;
                entry.guild_tag = character.guild_tag.clone().unwrap_or_default();
                entry.title = character.title.clone().unwrap_or_default();

                let in_party = self.get_player_party(*player_id).is_some();
                entry.icon = character.get_icon(in_party);
                online_list.push(entry);
            }
        }
        online_list
    }
}
