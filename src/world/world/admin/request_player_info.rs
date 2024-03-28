use eolib::protocol::net::{
    server::{
        AdminInteractTellServerPacket, BigCoords, CharacterBaseStats, CharacterElementalStats,
        CharacterSecondaryStatsInfoLookup, CharacterStatsInfoLookup,
    },
    PacketAction, PacketFamily,
};

use super::super::World;

impl World {
    // TODO: Work when player offline
    pub fn request_player_info(&mut self, player_id: i32, name: String) {
        let player = match self.players.get(&player_id) {
            Some(player) => player.to_owned(),
            None => return,
        };

        let target_player_id = match self.characters.get(&name) {
            Some(player_id) => player_id,
            None => return,
        };

        let target = match self.players.get(target_player_id) {
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
                PacketAction::Tell,
                PacketFamily::AdminInteract,
                &AdminInteractTellServerPacket {
                    name,
                    usage: character.usage,
                    exp: character.experience,
                    level: character.level,
                    map_id: character.map_id,
                    map_coords: BigCoords {
                        x: character.coords.x,
                        y: character.coords.y,
                    },
                    stats: CharacterStatsInfoLookup {
                        hp: character.hp,
                        max_hp: character.max_hp,
                        tp: character.tp,
                        max_tp: character.max_tp,
                        base_stats: CharacterBaseStats {
                            str: character.adj_strength,
                            intl: character.adj_intelligence,
                            wis: character.adj_wisdom,
                            agi: character.adj_agility,
                            con: character.adj_constitution,
                            cha: character.adj_charisma,
                        },
                        secondary_stats: CharacterSecondaryStatsInfoLookup {
                            min_damage: character.min_damage,
                            max_damage: character.max_damage,
                            accuracy: character.accuracy,
                            evade: character.evasion,
                            armor: character.armor,
                        },
                        elemental_stats: CharacterElementalStats::default(),
                    },
                    weight: character.get_weight(),
                },
            );
        });
    }
}
