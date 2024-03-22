use eolib::protocol::net::{
    server::AdminInteractListServerPacket, PacketAction, PacketFamily, ThreeItem,
};

use super::super::World;

impl World {
    // TODO: Work when player offline
    pub fn request_player_inventory(&mut self, player_id: i32, name: String) {
        let player = match self.players.get(&player_id) {
            Some(player) => player.to_owned(),
            None => return,
        };

        let target_player_id = match self.characters.get(&name) {
            Some(player_id) => player_id,
            None => return,
        };

        let target = match self.players.get(&target_player_id) {
            Some(player) => player.to_owned(),
            None => return,
        };

        tokio::spawn(async move {
            let character = match target.get_character().await {
                Ok(character) => character,
                Err(e) => {
                    error!("Failed to get character: {}", e);
                    return;
                }
            };

            player.send(
                PacketAction::List,
                PacketFamily::AdminInteract,
                &AdminInteractListServerPacket {
                    name,
                    usage: character.usage,
                    bank: character
                        .bank
                        .iter()
                        .map(|i| ThreeItem {
                            id: i.id,
                            amount: i.amount,
                        })
                        .collect(),
                    gold_bank: character.gold_bank,
                    inventory: character.items,
                },
            );
        });
    }
}
