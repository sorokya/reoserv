use std::cmp;

use evalexpr::{context_map, eval_float_with_context, DefaultNumericTypes, HashMapContext};

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

        let equipment_items = vec![
            self.equipment.boots,
            self.equipment.accessory,
            self.equipment.gloves,
            self.equipment.belt,
            self.equipment.armor,
            self.equipment.necklace,
            self.equipment.hat,
            self.equipment.shield,
            self.equipment.weapon,
            self.equipment.ring[0],
            self.equipment.ring[1],
            self.equipment.armlet[0],
            self.equipment.armlet[1],
            self.equipment.bracer[0],
            self.equipment.bracer[1],
        ];

        for item_id in equipment_items {
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

        let context: HashMapContext<DefaultNumericTypes> = match context_map! {
            "base_str" => int self.base_strength,
            "base_int" => int self.base_intelligence,
            "base_wis" => int self.base_wisdom,
            "base_agi" => int self.base_agility,
            "base_con" => int self.base_constitution,
            "base_cha" => int self.base_charisma,
            "str" => int self.adj_strength,
            "int" => int self.adj_intelligence,
            "wis" => int self.adj_wisdom,
            "agi" => int self.adj_agility,
            "con" => int self.adj_constitution,
            "cha" => int self.adj_charisma,
            "level" => int self.level,
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
