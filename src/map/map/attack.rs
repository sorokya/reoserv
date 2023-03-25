use chrono::Utc;
use eo::{
    data::{EOShort, Serializeable, StreamBuilder},
    protocol::{server::npc, Coords, Direction, PacketAction, PacketFamily},
};
use rand::Rng;

use crate::{
    map::{Item, Npc},
    DROP_DB,
};

use super::Map;

impl Map {
    pub fn attack(&mut self, target_player_id: EOShort, direction: Direction) {
        if let Some(target) = self.characters.get_mut(&target_player_id) {
            let target_coords = target.coords;
            let target_attack_coords = match direction {
                Direction::Up => Coords {
                    x: target_coords.x,
                    y: target_coords.y - 1,
                },
                Direction::Down => Coords {
                    x: target_coords.x,
                    y: target_coords.y + 1,
                },
                Direction::Left => Coords {
                    x: target_coords.x - 1,
                    y: target_coords.y,
                },
                Direction::Right => Coords {
                    x: target_coords.x + 1,
                    y: target_coords.y,
                },
            };

            if let Some((index, npc)) = self
                .npcs
                .iter_mut()
                .find(|(_, npc)| npc.coords == target_attack_coords && npc.alive)
            {
                npc.alive = false;
                npc.dead_since = Utc::now();

                let drop = get_drop(target_player_id, target_attack_coords, npc);

                let mut packet = npc::Spec {
                    killer_id: target_player_id,
                    killer_direction: direction.to_char(),
                    npc_index: *index as EOShort,
                    damage: npc.hp,
                    ..Default::default()
                };

                if let Some(drop) = drop {
                    let index = self.get_next_item_index(1);
                    packet.drop_index = index;
                    packet.drop_id = drop.id;
                    packet.drop_coords = target_attack_coords;
                    packet.drop_amount = drop.amount;
                    self.items.insert(index, drop);
                }

                debug!("{:?}", packet);

                let mut builder = StreamBuilder::new();
                packet.serialize(&mut builder);
                let buf = builder.get();

                for (_, character) in self.characters.iter() {
                    if character.is_in_range(&target_attack_coords) {
                        character.player.as_ref().unwrap().send(
                            PacketAction::Spec,
                            PacketFamily::Npc,
                            buf.clone(),
                        );
                    }
                }
            }
        }
    }
}

fn get_drop(target_player_id: EOShort, target_attack_coords: Coords, npc: &Npc) -> Option<Item> {
    if let Some(drop_npc) = DROP_DB.npcs.iter().find(|d| d.npc_id == npc.id) {
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
                    coords: target_attack_coords,
                    owner: target_player_id,
                });
            }
        }
    }

    None
}
