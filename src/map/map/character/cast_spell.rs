use std::cmp;

use eolib::protocol::net::server::{GroupHealTargetPlayer, SpellTargetGroupServerPacket};
use eolib::{
    protocol::{
        net::{
            server::{
                AvatarAdminServerPacket, RecoverPlayerServerPacket, SpellTargetOtherServerPacket,
                SpellTargetSelfServerPacket,
            },
            PacketAction, PacketFamily,
        },
        r#pub::{EsfRecord, NpcType, SkillTargetRestrict, SkillTargetType, SkillType},
    },
};
use rand::Rng;

use crate::utils::in_client_range;
use crate::{
    character::{SpellState, SpellTarget},
    NPC_DB, SETTINGS, SPELL_DB,
};

use super::super::Map;

impl Map {
    pub async fn cast_spell(&mut self, player_id: i32, target: SpellTarget) {
        let spell_id = match self.get_player_spell_id(player_id) {
            Some(spell_id) => spell_id,
            None => return,
        };

        let spell_data = match SPELL_DB.skills.get(spell_id as usize - 1) {
            Some(spell_data) => spell_data,
            None => return,
        };

        match spell_data.r#type {
            SkillType::Heal => {
                self.cast_heal_spell(player_id, spell_id, spell_data, target)
                    .await
            }
            SkillType::Attack => {
                self.cast_damage_spell(player_id, spell_id, spell_data, target)
                    .await
            }
            SkillType::Bard => {}
            _ => {}
        }
    }

    fn get_player_spell_id(&self, player_id: i32) -> Option<i32> {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return None,
        };

        match character.spell_state {
            SpellState::Requested {
                spell_id,
                timestamp: _,
                cast_time: _,
            } => {
                // TODO: enforce timestamp
                if character.has_spell(spell_id) {
                    Some(spell_id)
                } else {
                    None
                }
            }
            SpellState::None => None,
        }
    }

    async fn cast_heal_spell(
        &mut self,
        player_id: i32,
        spell_id: i32,
        spell: &EsfRecord,
        target: SpellTarget,
    ) {
        if spell.target_restrict != SkillTargetRestrict::Friendly {
            return;
        }
        match target {
            SpellTarget::Player => self.cast_heal_self(player_id, spell_id, spell),
            SpellTarget::Group => self.cast_heal_group(player_id, spell_id, spell).await,
            SpellTarget::OtherPlayer(target_player_id) => {
                self.cast_heal_player(player_id, target_player_id, spell_id, spell);
            }
            _ => {}
        }
    }
    fn cast_heal_self(&mut self, player_id: i32, spell_id: i32, spell: &EsfRecord) {
        if spell.target_type != SkillTargetType::SELF {
            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        if character.tp < spell.tp_cost {
            return;
        }

        character.spell_state = SpellState::None;
        character.tp -= spell.tp_cost;
        let original_hp = character.hp;
        character.hp = cmp::min(character.hp + spell.hp_heal, character.max_hp);

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let hp_percentage = character.get_hp_percentage();

        if character.hp != original_hp {
            character
                .player
                .as_ref()
                .unwrap()
                .update_party_hp(hp_percentage);
        }

        self.send_packet_near_player(
            player_id,
            PacketAction::TargetSelf,
            PacketFamily::Spell,
            &SpellTargetSelfServerPacket {
                player_id,
                spell_id,
                spell_heal_hp: spell.hp_heal,
                hp_percentage,
                hp: None,
                tp: None,
            },
        );

        character.player.as_ref().unwrap().send(
            PacketAction::TargetSelf,
            PacketFamily::Spell,
            &SpellTargetSelfServerPacket {
                player_id,
                spell_id,
                spell_heal_hp: spell.hp_heal,
                hp_percentage,
                hp: Some(character.hp),
                tp: Some(character.tp),
            },
        );
    }

    async fn cast_heal_group(&mut self, player_id: i32, spell_id: i32, spell: &EsfRecord) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        if character.tp < spell.tp_cost {
            return;
        }

        let party_player_ids = match self.world.get_player_party(player_id).await {
            Some(party) => party.members,
            None => return,
        };

        character.spell_state = SpellState::None;
        character.tp -= spell.tp_cost;

        let mut healed_players: Vec<GroupHealTargetPlayer> =
            Vec::with_capacity(party_player_ids.len());

        for party_member_id in party_player_ids {
            let member_character = match self.characters.get_mut(&party_member_id) {
                Some(character) => character,
                None => continue,
            };

            let original_hp = member_character.hp;
            member_character.hp =
                cmp::min(member_character.hp + spell.hp_heal, member_character.max_hp);
            let hp_percentage = member_character.get_hp_percentage();

            if member_character.hp != original_hp {
                member_character
                    .player
                    .as_ref()
                    .unwrap()
                    .update_party_hp(hp_percentage);
            }

            healed_players.push(GroupHealTargetPlayer {
                player_id: party_member_id,
                hp_percentage,
                hp: member_character.hp,
            });
        }

        for character in self.characters.values() {
            let in_range_healed_players = healed_players
                .iter()
                .filter_map(|healed| {
                    let healed_character = match self.characters.get(&healed.player_id) {
                        Some(character) => character,
                        None => return None,
                    };

                    if player_id == healed.player_id
                        || in_client_range(&character.coords, &healed_character.coords)
                    {
                        Some(healed.to_owned())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            if !in_range_healed_players.is_empty() {
                let packet = SpellTargetGroupServerPacket {
                    spell_id,
                    caster_id: player_id,
                    caster_tp: character.tp,
                    spell_heal_hp: spell.hp_heal,
                    players: in_range_healed_players,
                };

                character.player.as_ref().unwrap().send(
                    PacketAction::TargetGroup,
                    PacketFamily::Spell,
                    &packet,
                );
            }
        }
    }

    fn cast_heal_player(
        &mut self,
        player_id: i32,
        target_player_id: i32,
        spell_id: i32,
        spell: &EsfRecord,
    ) {
        if spell.target_type != SkillTargetType::Normal {
            return;
        }

        if !self.characters.contains_key(&target_player_id) {
            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        if character.tp < spell.tp_cost {
            return;
        }

        character.spell_state = SpellState::None;
        character.tp -= spell.tp_cost;

        let target = match self.characters.get_mut(&target_player_id) {
            Some(character) => character,
            None => return,
        };

        let original_hp = target.hp;
        target.hp = cmp::min(target.hp + spell.hp_heal, target.max_hp);
        let hp_percentage = target.get_hp_percentage();

        if target.hp != original_hp {
            target
                .player
                .as_ref()
                .unwrap()
                .update_party_hp(hp_percentage);
        }

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let target = match self.characters.get(&target_player_id) {
            Some(character) => character,
            None => return,
        };

        let mut packet = SpellTargetOtherServerPacket {
            victim_id: target_player_id,
            caster_id: player_id,
            caster_direction: character.direction,
            spell_id,
            spell_heal_hp: spell.hp_heal,
            hp_percentage: target.get_hp_percentage(),
            hp: None,
        };

        self.send_packet_near_player(
            target_player_id,
            PacketAction::TargetOther,
            PacketFamily::Spell,
            &packet,
        );

        packet.hp = Some(target.hp);

        target.player.as_ref().unwrap().send(
            PacketAction::TargetOther,
            PacketFamily::Spell,
            &packet,
        );

        let packet = RecoverPlayerServerPacket {
            hp: character.hp,
            tp: character.tp,
        };

        character.player.as_ref().unwrap().send(
            PacketAction::Player,
            PacketFamily::Recover,
            &packet,
        );
    }

    async fn cast_damage_spell(
        &mut self,
        player_id: i32,
        spell_id: i32,
        spell_data: &EsfRecord,
        target: SpellTarget,
    ) {
        if spell_data.target_restrict == SkillTargetRestrict::Friendly
            || spell_data.target_type != SkillTargetType::Normal
        {
            return;
        }

        match target {
            SpellTarget::Npc(npc_index) => {
                self.cast_damage_npc(player_id, npc_index, spell_id, spell_data)
                    .await
            }
            SpellTarget::OtherPlayer(target_player_id) => {
                self.cast_damage_player(player_id, target_player_id, spell_id, spell_data)
                    .await
            }
            _ => {}
        }
    }

    async fn cast_damage_npc(
        &mut self,
        player_id: i32,
        npc_index: i32,
        spell_id: i32,
        spell_data: &EsfRecord,
    ) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        if character.tp < spell_data.tp_cost {
            return;
        }

        let direction = character.direction;

        let npc = match self.npcs.get_mut(&npc_index) {
            Some(npc) => npc,
            None => return,
        };

        let npc_data = match NPC_DB.npcs.get(npc.id as usize - 1) {
            Some(npc_data) => npc_data,
            None => return,
        };

        if !matches!(npc_data.r#type, NpcType::Passive | NpcType::Aggressive) {
            return;
        }

        character.spell_state = SpellState::None;
        character.tp -= spell_data.tp_cost;

        let party_player_ids = match self.world.get_player_party(player_id).await {
            Some(party) => party.members,
            None => Vec::new(),
        };

        let protected = npc_data.behavior_id == 0
            && !npc.opponents.is_empty()
            && !npc.opponents.iter().any(|o| {
                o.player_id == player_id
                    || party_player_ids.contains(&o.player_id)
                    || o.bored_ticks >= SETTINGS.npcs.bored_timer
            });

        let damage_dealt = if protected {
            0
        } else {
            let amount = {
                let mut rng = rand::thread_rng();
                rng.gen_range(
                    character.min_damage + spell_data.min_damage
                        ..=character.max_damage + spell_data.max_damage,
                )
            };

            let critical = npc.hp == npc.max_hp;

            npc.damage(player_id, amount, character.accuracy, critical)
        };

        character.player.as_ref().unwrap().send(
            PacketAction::Player,
            PacketFamily::Recover,
            &RecoverPlayerServerPacket {
                hp: character.hp,
                tp: character.tp,
            },
        );

        if npc.alive {
            self.attack_npc_reply(
                player_id,
                npc_index,
                direction,
                damage_dealt,
                Some(spell_id),
                protected,
            );
        } else {
            self.attack_npc_killed_reply(player_id, npc_index, damage_dealt, Some(spell_id))
                .await;
        }
    }

    async fn cast_damage_player(
        &mut self,
        player_id: i32,
        target_player_id: i32,
        spell_id: i32,
        spell_data: &EsfRecord,
    ) {
        let (tp, direction, min_damage, max_damage, accuracy) =
            match self.characters.get(&player_id) {
                Some(character) => (
                    character.tp,
                    character.direction,
                    character.min_damage,
                    character.max_damage,
                    character.accuracy,
                ),
                None => return,
            };

        if tp < spell_data.tp_cost {
            return;
        }

        let amount = {
            let mut rng = rand::thread_rng();
            rng.gen_range(min_damage + spell_data.min_damage..=max_damage + spell_data.max_damage)
        };

        let damage_dealt = {
            let target_character = match self.characters.get_mut(&target_player_id) {
                Some(character) => character,
                None => return,
            };

            let critical = target_character.hp == target_character.max_hp;

            target_character.damage(amount, accuracy, critical)
        };

        {
            let character = match self.characters.get_mut(&player_id) {
                Some(character) => character,
                None => return,
            };

            character.spell_state = SpellState::None;
            character.tp -= spell_data.tp_cost;

            character.player.as_ref().unwrap().send(
                PacketAction::Player,
                PacketFamily::Recover,
                &RecoverPlayerServerPacket {
                    hp: character.hp,
                    tp: character.tp,
                },
            );
        }

        let target_character = match self.characters.get(&target_player_id) {
            Some(character) => character,
            None => return,
        };

        let packet = AvatarAdminServerPacket {
            caster_id: player_id,
            victim_id: target_player_id,
            caster_direction: direction,
            damage: damage_dealt,
            hp_percentage: target_character.get_hp_percentage(),
            victim_died: target_character.hp == 0,
            spell_id,
        };

        self.send_packet_near(
            &target_character.coords,
            PacketAction::Admin,
            PacketFamily::Avatar,
            packet,
        );

        if target_character.hp == 0 {
            target_character.player.as_ref().unwrap().die();
        }

        target_character.player.as_ref().unwrap().send(
            PacketAction::Player,
            PacketFamily::Recover,
            &RecoverPlayerServerPacket {
                hp: target_character.hp,
                tp: target_character.tp,
            },
        );

        target_character
            .player
            .as_ref()
            .unwrap()
            .update_party_hp(target_character.get_hp_percentage());
    }
}
