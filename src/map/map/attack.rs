use std::cmp;

use chrono::Utc;
use eo::{
    data::{EOInt, EOShort},
    protocol::{server::npc, Coords, Direction, PacketAction, PacketFamily},
};
use evalexpr::{context_map, eval_float_with_context};
use rand::Rng;

use crate::{
    character::Character,
    map::{Item, Npc},
    DROP_DB, FORMULAS, NPC_DB,
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

            let (index, damage) = if let Some((index, npc)) = self
                .npcs
                .iter()
                .find(|(_, npc)| npc.coords == target_attack_coords && npc.alive)
            {
                (*index, get_damage_amount(target, npc))
            } else {
                return;
            };

            let killed = {
                let npc = self.npcs.get_mut(&index).unwrap();
                npc.hp -= damage;
                npc.hp == 0
            };

            if killed {
                {
                    let npc = self.npcs.get_mut(&index).unwrap();
                    npc.alive = false;
                    npc.dead_since = Utc::now();
                }

                let drop = {
                    let npc = self.npcs.get(&index).unwrap();
                    get_drop(target_player_id, target_attack_coords, npc)
                };

                let mut packet = npc::Spec {
                    killer_id: target_player_id,
                    killer_direction: direction.to_char(),
                    npc_index: index as EOShort,
                    damage,
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

                self.send_packet_near(
                    &target_attack_coords,
                    PacketAction::Spec,
                    PacketFamily::Npc,
                    packet,
                );

                return;
            }

            let npc = self.npcs.get(&index).unwrap();
            let reply = npc::Reply {
                player_id: target_player_id,
                npc_index: index as EOShort,
                damage,
                player_direction: target.direction.to_char(),
                hp_percentage: npc.get_hp_percentage() as EOShort,
            };

            debug!("{:?}", reply);

            self.send_packet_near(
                &target_attack_coords,
                PacketAction::Reply,
                PacketFamily::Npc,
                reply,
            );
        }
    }
}

fn get_damage_amount(character: &Character, npc: &Npc) -> EOInt {
    let mut rng = rand::thread_rng();
    let rand = rng.gen_range(0.0..=1.0);

    let amount = rng.gen_range(character.min_damage..=character.max_damage);

    let player_facing_npc =
        ((npc.direction.to_char() as i32) - (character.direction.to_char() as i32)).abs() != 2;

    let critical = npc.hp == npc.max_hp || player_facing_npc;

    let npc_data = match NPC_DB.npcs.get(npc.id as usize) {
        Some(npc_data) => npc_data,
        None => {
            error!("Failed to find npc data for npc id {}", npc.id);
            return 0;
        }
    };

    let context = match context_map! {
        "critical" => critical,
        "damage" => amount as f64,
        "target_armor" => npc_data.armor as f64,
        "target_sitting" => false,
        "accuracy" => character.accuracy as f64,
        "target_evade" => npc_data.evade as f64,
    } {
        Ok(context) => context,
        Err(e) => {
            error!("Failed to generate formula context: {}", e);
            return 0;
        }
    };

    let hit_rate = match eval_float_with_context(&FORMULAS.hit_rate, &context) {
        Ok(hit_rate) => hit_rate,
        Err(e) => {
            error!("Failed to calculate hit rate: {}", e);
            0.0
        }
    };

    if hit_rate < rand {
        return 0;
    }

    match eval_float_with_context(&FORMULAS.damage, &context) {
        Ok(amount) => cmp::min(amount.floor() as EOInt, npc.hp as EOInt),
        Err(e) => {
            error!("Failed to calculate damage: {}", e);
            0
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
