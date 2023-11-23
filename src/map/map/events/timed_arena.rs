use bytes::Bytes;
use eo::{
    data::{EOChar, StreamBuilder},
    protocol::{Coords, PacketAction, PacketFamily},
};

use crate::{map::map::ArenaPlayer, ARENAS, SETTINGS};

use super::super::Map;

impl Map {
    pub fn timed_arena(&mut self) {
        let config = match ARENAS.arenas.iter().find(|a| a.map == self.id) {
            Some(config) => config,
            None => return,
        };

        self.arena_ticks += SETTINGS.world.arena_spawn_rate;

        if self.arena_ticks >= config.rate {
            self.arena_ticks = 0;

            if self.arena_players.len() as EOChar >= config.block {
                return self.send_arena_full();
            }

            let mut queued_characters: Vec<ArenaPlayer> = self
                .characters
                .values()
                .into_iter()
                .filter(|c| {
                    config
                        .spawns
                        .iter()
                        .any(|s| s.from.x == c.coords.x && s.from.y == c.coords.y)
                })
                .map(|c| ArenaPlayer {
                    player_id: c.player_id.unwrap(),
                    kills: 0,
                })
                .collect();

            if queued_characters.len() == 0
                || (self.arena_players.len() == 0 && queued_characters.len() == 1)
            {
                return;
            }

            self.send_arena_launch(queued_characters.len());

            for arena_player in &queued_characters {
                let character = match self.characters.get(&arena_player.player_id) {
                    Some(character) => character,
                    None => continue,
                };

                let spawn = match config
                    .spawns
                    .iter()
                    .find(|s| s.from.x == character.coords.x && s.from.y == character.coords.y)
                {
                    Some(spawn) => spawn,
                    None => continue,
                };

                character.player.as_ref().unwrap().request_warp(
                    self.id,
                    Coords {
                        x: spawn.to.x,
                        y: spawn.to.y,
                    },
                    true,
                    None,
                );
            }

            self.arena_players.append(&mut queued_characters);
        }
    }

    fn send_arena_full(&self) {
        let buf = Bytes::from_static(&b"N"[..]);
        for character in self.characters.values() {
            character.player.as_ref().unwrap().send(
                PacketAction::Drop,
                PacketFamily::Arena,
                buf.clone(),
            );
        }
    }

    fn send_arena_launch(&mut self, player_count: usize) {
        let mut builder = StreamBuilder::new();
        builder.add_char(player_count as EOChar);

        let buf = builder.get();
        for character in self.characters.values() {
            character.player.as_ref().unwrap().send(
                PacketAction::Use,
                PacketFamily::Arena,
                buf.clone(),
            );
        }
    }
}
