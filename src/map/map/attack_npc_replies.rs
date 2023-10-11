use eo::{
    data::{EOChar, EOInt, EOShort, StreamBuilder},
    protocol::{server::attack, Coords, Direction, PacketAction, PacketFamily},
};
use rand::Rng;

use crate::{map::Item, DROP_DB, NPC_DB};

use super::Map;

impl Map {
    pub fn attack_npc_reply(
        &mut self,
        player_id: EOShort,
        npc_index: EOChar,
        direction: Direction,
        damage_dealt: EOInt,
        spell_id: Option<EOShort>,
    ) {
        if spell_id.is_none() {
            let reply = attack::Player {
                player_id,
                direction,
            };

            self.send_packet_near_player(
                player_id,
                PacketAction::Player,
                PacketFamily::Attack,
                reply,
            );
        }

        let npc = match self.npcs.get(&npc_index) {
            Some(npc) => npc,
            None => return,
        };

        let mut builder = StreamBuilder::new();
        if let Some(spell_id) = spell_id {
            builder.add_short(spell_id);
        }

        builder.add_short(player_id);
        builder.add_char(direction.to_char());
        builder.add_short(npc_index as EOShort);
        builder.add_three(damage_dealt);
        builder.add_short(npc.get_hp_percentage() as EOShort);

        if spell_id.is_some() {
            let tp = match self.characters.get(&player_id) {
                Some(character) => character.tp,
                None => 0,
            };
            builder.add_short(tp);
        }

        self.send_buf_near(
            &npc.coords,
            PacketAction::Reply,
            if spell_id.is_some() {
                PacketFamily::Cast
            } else {
                PacketFamily::Npc
            },
            builder.get(),
        );
    }

    pub fn attack_npc_killed_reply(
        &mut self,
        killer_player_id: EOShort,
        npc_index: EOChar,
        direction: Direction,
        damage_dealt: EOInt,
        spell_id: Option<EOShort>,
    ) {
        let (npc_id, npc_coords) = match self.npcs.get(&npc_index) {
            Some(npc) => (npc.id, npc.coords),
            None => return,
        };

        let npc_data = match NPC_DB.npcs.get(npc_id as usize - 1) {
            Some(npc_data) => npc_data,
            None => return,
        };

        // TODO: Party experience
        let (leveled_up, experience) = self.give_experience(killer_player_id, npc_data.experience);

        let drop = get_drop(killer_player_id, npc_id, &npc_coords);

        let (drop_index, drop_item_id, drop_amount) = match drop {
            Some(drop) => {
                let index = self.get_next_item_index(1);
                let drop_item_id = drop.id;
                let drop_amount = drop.amount;
                self.items.insert(index, drop);
                (index, drop_item_id, drop_amount)
            }
            None => (0, 0, 0),
        };

        for (player_id, character) in self.characters.iter() {
            let mut builder = StreamBuilder::new();
            if let Some(spell_id) = spell_id {
                builder.add_short(spell_id);
            }

            builder.add_short(killer_player_id);
            builder.add_char(direction.to_char());
            builder.add_short(npc_index as EOShort);
            builder.add_short(drop_index);
            builder.add_short(drop_item_id);
            builder.add_char(npc_coords.x);
            builder.add_char(npc_coords.y);
            builder.add_int(drop_amount);
            builder.add_three(damage_dealt);

            if spell_id.is_some() {
                let tp = match self.characters.get(&killer_player_id) {
                    Some(character) => character.tp,
                    None => 0,
                };
                builder.add_short(tp);
            }

            if player_id == &killer_player_id {
                builder.add_int(experience);

                if leveled_up {
                    let character = match self.characters.get(&killer_player_id) {
                        Some(character) => character,
                        None => return,
                    };

                    builder.add_char(character.level);
                    builder.add_short(character.stat_points);
                    builder.add_short(character.skill_points);
                    builder.add_short(character.max_hp);
                    builder.add_short(character.max_tp);
                    builder.add_short(character.max_sp);
                }
            }

            let family = match spell_id {
                Some(_) => PacketFamily::Cast,
                _ => PacketFamily::Npc,
            };

            let action = match player_id == &killer_player_id && leveled_up {
                true => PacketAction::Accept,
                _ => PacketAction::Spec,
            };

            character
                .player
                .as_ref()
                .unwrap()
                .send(action, family, builder.get());
        }
    }
}

fn get_drop(target_player_id: EOShort, npc_id: EOShort, npc_coords: &Coords) -> Option<Item> {
    if let Some(drop_npc) = DROP_DB.npcs.iter().find(|d| d.npc_id == npc_id) {
        let mut rng = rand::thread_rng();
        let mut drops = drop_npc.drops.clone();
        drops.sort_by(|a, b| a.rate.cmp(&b.rate));

        for drop in drops {
            let roll = rng.gen_range(0..=64000);
            if roll <= drop.rate {
                let amount = rng.gen_range(drop.min..=drop.max);
                return Some(Item {
                    id: drop.item_id,
                    amount,
                    coords: *npc_coords,
                    owner: target_player_id,
                });
            }
        }
    }

    None
}
