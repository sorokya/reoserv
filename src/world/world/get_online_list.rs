use eo::protocol::OnlinePlayers;

use super::World;

impl World {
    pub async fn get_online_list(&self) -> Vec<OnlinePlayers> {
        let mut online_list = Vec::new();
        for player in self.players.values() {
            if let Ok(character) = player.get_character().await {
                if character.hidden {
                    continue;
                }

                let mut entry = OnlinePlayers::new();
                entry.name = character.name.to_string();
                entry.class_id = character.class;
                entry.guild_tag = character.guild_tag.clone().unwrap_or_default();
                entry.title = character.title.clone().unwrap_or_default();
                entry.icon = character.get_icon();
                online_list.push(entry);
            }
        }
        online_list
    }
}
