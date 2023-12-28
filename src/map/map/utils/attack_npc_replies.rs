use eolib::{protocol::{Direction, net::{server::AttackPlayerServerPacket, PacketAction, PacketFamily}, Coords}, data::EoWriter};
use evalexpr::{context_map, eval_float_with_context};
use rand::Rng;

use crate::{map::Item, DROP_DB, FORMULAS, NPC_DB};

use super::super::Map;

impl Map {
    pub fn attack_npc_reply(
        &mut self,
        player_id: i32,
        npc_index: i32,
        direction: Direction,
        damage_dealt: i32,
        spell_id: Option<i32>,
    ) {
        if spell_id.is_none() {
            let reply = AttackPlayerServerPacket {
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

        let mut writer = EoWriter::new();
        if let Some(spell_id) = spell_id {
            writer.add_short(spell_id);
        }

        writer.add_short(player_id);
        writer.add_char(i32::from(direction));
        writer.add_short(npc_index);
        writer.add_three(damage_dealt);
        writer.add_short(npc.get_hp_percentage());

        if spell_id.is_some() {
            let tp = match self.characters.get(&player_id) {
                Some(character) => character.tp,
                None => 0,
            };
            writer.add_short(tp);
        }

        self.send_buf_near(
            &npc.coords,
            PacketAction::Reply,
            if spell_id.is_some() {
                PacketFamily::Cast
            } else {
                PacketFamily::Npc
            },
            writer.to_byte_array(),
        );
    }

    pub async fn attack_npc_killed_reply(
        &mut self,
        killer_player_id: i32,
        npc_index: i32,
        direction: Direction,
        damage_dealt: i32,
        spell_id: Option<i32>,
    ) {
        let (npc_id, npc_coords) = match self.npcs.get(&npc_index) {
            Some(npc) => (npc.id, npc.coords),
            None => return,
        };

        let npc_data = match NPC_DB.npcs.get(npc_id as usize - 1) {
            Some(npc_data) => npc_data,
            None => return,
        };

        let mut exp_gains: Vec<(i32, bool, i32, i32)> = Vec::new();

        if let Some(party) = self.world.get_player_party(killer_player_id).await {
            let members_on_map: Vec<&i32> = party
                .members
                .iter()
                .filter(|id| self.characters.contains_key(id))
                .collect();

            let experience = if members_on_map.len() > 1 {
                let context = match context_map! {
                    "members" => members_on_map.len() as f64,
                    "exp" => npc_data.experience as f64,
                } {
                    Ok(context) => context,
                    Err(e) => {
                        error!("Failed to generate formula context: {}", e);
                        return;
                    }
                };

                match eval_float_with_context(&FORMULAS.party_exp_share, &context) {
                    Ok(experience) => experience as i32,
                    Err(e) => {
                        error!("Failed to calculate party experience share: {}", e);
                        1
                    }
                }
            } else {
                npc_data.experience
            };

            for member_id in members_on_map {
                let (leveled_up, total_experience, experience_gained) =
                    self.give_experience(*member_id, experience);
                exp_gains.push((*member_id, leveled_up, total_experience, experience_gained));
            }
        } else {
            let (leveled_up, experience, experience_gained) =
                self.give_experience(killer_player_id, npc_data.experience);
            exp_gains.push((killer_player_id, leveled_up, experience, experience_gained));
        }

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
            let mut writer = EoWriter::new();
            if let Some(spell_id) = spell_id {
                writer.add_short(spell_id);
            }

            writer.add_short(killer_player_id);
            writer.add_char(i32::from(direction));
            writer.add_short(npc_index);
            writer.add_short(drop_index);
            writer.add_short(drop_item_id);
            writer.add_char(npc_coords.x);
            writer.add_char(npc_coords.y);
            writer.add_int(drop_amount);
            writer.add_three(damage_dealt);

            if spell_id.is_some() {
                let tp = match self.characters.get(&killer_player_id) {
                    Some(character) => character.tp,
                    None => 0,
                };
                writer.add_short(tp);
            }

            let leveled_up = if let Some((_, leveled_up, total_experience, _)) =
                exp_gains.iter().find(|(id, _, _, _)| id == player_id)
            {
                if exp_gains.len() == 1 {
                    writer.add_int(*total_experience);

                    if *leveled_up {
                        let character = match self.characters.get(&killer_player_id) {
                            Some(character) => character,
                            None => return,
                        };

                        writer.add_char(character.level);
                        writer.add_short(character.stat_points);
                        writer.add_short(character.skill_points);
                        writer.add_short(character.max_hp);
                        writer.add_short(character.max_tp);
                        writer.add_short(character.max_sp);
                    }
                }

                *leveled_up
            } else {
                false
            };

            let family = match spell_id {
                Some(_) => PacketFamily::Cast,
                _ => PacketFamily::Npc,
            };

            let action = match exp_gains.len() == 1 && leveled_up {
                true => PacketAction::Accept,
                _ => PacketAction::Spec,
            };

            character
                .player
                .as_ref()
                .unwrap()
                .send(action, family, writer.to_byte_array());
        }

        if exp_gains.len() > 1 {
            self.attack_npc_killed_party_reply(&exp_gains);
        }
    }

    fn attack_npc_killed_party_reply(&self, exp_gains: &Vec<(i32, bool, i32, i32)>) {
        for (player_id, leveled_up, _, experience) in exp_gains {
            let character = match self.characters.get(player_id) {
                Some(character) => character,
                None => continue,
            };

            if *leveled_up {
                let mut writer = EoWriter::new();
                writer.add_short(character.stat_points);
                writer.add_short(character.skill_points);
                writer.add_short(character.max_hp);
                writer.add_short(character.max_tp);
                writer.add_short(character.max_sp);

                character.player.as_ref().unwrap().send(
                    PacketAction::TargetGroup,
                    PacketFamily::Recover,
                    writer.to_byte_array(),
                );

                let mut writer = EoWriter::new();
                writer.add_int(character.experience);
                writer.add_short(character.karma);
                writer.add_char(1);
                writer.add_short(character.stat_points);
                writer.add_short(character.skill_points);

                character.player.as_ref().unwrap().send(
                    PacketAction::Reply,
                    PacketFamily::Recover,
                    writer.to_byte_array(),
                );
            }

            let mut writer = EoWriter::new();
            writer.add_short(*player_id);
            writer.add_int(*experience);
            writer.add_char(if *leveled_up { 1 } else { 0 });

            self.send_buf_near(
                &character.coords,
                PacketAction::TargetGroup,
                PacketFamily::Party,
                writer.to_byte_array(),
            );
        }
    }
}

fn get_drop(target_player_id: i32, npc_id: i32, npc_coords: &Coords) -> Option<Item> {
    if let Some(drop_npc) = DROP_DB.npcs.iter().find(|d| d.npc_id == npc_id) {
        let mut rng = rand::thread_rng();
        let mut drops = drop_npc.drops.clone();
        drops.sort_by(|a, b| a.rate.cmp(&b.rate));

        for drop in drops {
            let roll = rng.gen_range(0..=64000);
            if roll <= drop.rate {
                let amount = rng.gen_range(drop.min_amount..=drop.max_amount);
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
