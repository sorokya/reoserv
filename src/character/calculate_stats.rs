use std::cmp;

use evalexpr::{context_map, eval_float_with_context};

use crate::{CLASS_DB, FORMULAS, ITEM_DB};

use super::Character;

impl Character {
    pub fn calculate_stats(&mut self) {
        let original_hp = self.hp;
        let original_max_hp = self.max_hp;

        let class = &CLASS_DB.classes[(self.class - 1) as usize];

        self.adj_strength = self.base_strength + class.str;
        self.adj_intelligence = self.base_intelligence + class.intl;
        self.adj_wisdom = self.base_wisdom + class.wis;
        self.adj_agility = self.base_agility + class.agi;
        self.adj_constitution = self.base_constitution + class.con;
        self.adj_charisma = self.base_charisma + class.cha;

        self.weight = 0;
        self.max_hp = 0;
        self.max_tp = 0;
        self.min_damage = 0;
        self.max_damage = 0;
        self.accuracy = 0;
        self.evasion = 0;
        self.armor = 0;
        self.max_sp = 0;

        for item in &self.items {
            if item.id == 0 {
                continue;
            }

            let record = &ITEM_DB.items[(item.id - 1) as usize];
            self.weight += record.weight * item.amount;
        }

        let paperdoll_items = vec![
            self.paperdoll.boots,
            self.paperdoll.accessory,
            self.paperdoll.gloves,
            self.paperdoll.belt,
            self.paperdoll.armor,
            self.paperdoll.necklace,
            self.paperdoll.hat,
            self.paperdoll.shield,
            self.paperdoll.weapon,
            self.paperdoll.ring[0],
            self.paperdoll.ring[1],
            self.paperdoll.armlet[0],
            self.paperdoll.armlet[1],
            self.paperdoll.bracer[0],
            self.paperdoll.bracer[1],
        ];

        for item_id in paperdoll_items {
            if item_id == 0 {
                continue;
            }

            let item = &ITEM_DB.items[(item_id - 1) as usize];
            self.weight += item.weight;
            self.max_hp += item.hp;
            self.max_tp += item.tp;
            self.min_damage += item.min_damage;
            self.max_damage += item.max_damage;
            self.accuracy += item.accuracy;
            self.evasion += item.evade;
            self.armor += item.armor;
            self.adj_strength += item.str;
            self.adj_intelligence += item.intl;
            self.adj_wisdom += item.wis;
            self.adj_agility += item.agi;
            self.adj_constitution += item.con;
            self.adj_charisma += item.cha;
        }

        let context = match context_map! {
            "base_str" => self.base_strength as i64,
            "base_int" => self.base_intelligence as i64,
            "base_wis" => self.base_wisdom as i64,
            "base_agi" => self.base_agility as i64,
            "base_con" => self.base_constitution as i64,
            "base_cha" => self.base_charisma as i64,
            "str" => self.adj_strength as i64,
            "int" => self.adj_intelligence as i64,
            "wis" => self.adj_wisdom as i64,
            "agi" => self.adj_agility as i64,
            "con" => self.adj_constitution as i64,
            "cha" => self.adj_charisma as i64,
            "level" => self.level as i64,
        } {
            Ok(context) => context,
            Err(e) => {
                error!("Failed to generate formula context: {}", e);
                return;
            }
        };

        self.max_hp += match eval_float_with_context(&FORMULAS.hp, &context) {
            Ok(max_hp) => cmp::min(max_hp.floor() as i32, 64000),
            Err(e) => {
                error!("Failed to calculate max_hp: {}", e);
                10
            }
        };

        self.max_tp += match eval_float_with_context(&FORMULAS.tp, &context) {
            Ok(max_tp) => cmp::min(max_tp.floor() as i32, 64000),
            Err(e) => {
                error!("Failed to calculate max_tp: {}", e);
                10
            }
        };

        self.max_sp += match eval_float_with_context(&FORMULAS.sp, &context) {
            Ok(max_sp) => cmp::min(max_sp.floor() as i32, 64000),
            Err(e) => {
                error!("Failed to calculate max_sp: {}", e);
                20
            }
        };

        self.max_weight = match eval_float_with_context(&FORMULAS.max_weight, &context) {
            Ok(max_weight) => cmp::min(max_weight.floor() as i32, 250),
            Err(e) => {
                error!("Failed to calculate max_weight: {}", e);
                70
            }
        };

        let class_formulas = &FORMULAS.classes[class.stat_group as usize];
        let damage = match eval_float_with_context(&class_formulas.damage, &context) {
            Ok(damage) => damage.floor() as i32,
            Err(e) => {
                error!("Failed to calculate damage: {}", e);
                1
            }
        };

        self.min_damage += damage;
        self.max_damage += damage;

        self.accuracy += match eval_float_with_context(&class_formulas.accuracy, &context) {
            Ok(accuracy) => accuracy.floor() as i32,
            Err(e) => {
                error!("Failed to calculate accuracy: {}", e);
                0
            }
        };

        self.armor += match eval_float_with_context(&class_formulas.defense, &context) {
            Ok(armor) => armor.floor() as i32,
            Err(e) => {
                error!("Failed to calculate armor: {}", e);
                0
            }
        };

        self.evasion += match eval_float_with_context(&class_formulas.evade, &context) {
            Ok(evasion) => evasion.floor() as i32,
            Err(e) => {
                error!("Failed to calculate evasion: {}", e);
                0
            }
        };

        if self.min_damage == 0 {
            self.min_damage = 1;
        }

        if self.max_damage == 0 {
            self.max_damage = 1;
        }

        if self.hp > self.max_hp {
            self.hp = self.max_hp;
        }

        if self.tp > self.max_tp {
            self.tp = self.max_tp;
        }

        if self.hp != original_hp || self.max_hp != original_max_hp {
            self.player
                .as_ref()
                .unwrap()
                .update_party_hp(self.get_hp_percentage());
        }
    }
}
