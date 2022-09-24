use eo::data::{pubs::{ItemFile, ClassFile}, EOInt, EOShort};
use evalexpr::{context_map, eval_float_with_context};

use crate::{character::Character, FORMULAS};

pub fn calculate_stats (character: &mut Character, item_file: &ItemFile, class_file: &ClassFile) {
    let class = &class_file.records[(character.class - 1) as usize];

    character.adj_strength = character.base_strength + class.strength;
    character.adj_intelligence = character.base_intelligence + class.intelligence;
    character.adj_wisdom = character.base_wisdom + class.wisdom;
    character.adj_agility = character.base_agility + class.agility;
    character.adj_constitution = character.base_constitution + class.constitution;
    character.adj_charisma = character.base_charisma + class.charisma;

    character.weight = 0;
    character.max_hp = 0;
    character.max_tp = 0;
    character.min_damage = 0;
    character.max_damage = 0;
    character.accuracy = 0;
    character.evasion = 0;
    character.armor = 0;
    character.max_sp = 0;

    for item in &character.items {
        if item.id == 0 {
            continue;
        }

        let record = &item_file.records[item.id as usize];
        character.weight += record.weight as EOInt * item.amount;
        if character.weight >= 250 {
            break;
        }
    }

    for item_id in character.paperdoll {
        if item_id == 0 {
            continue;
        }

        let item = &item_file.records[item_id as usize];
        character.weight += item.weight as EOInt;
        character.max_hp += item.hp;
        character.max_tp += item.tp;
        character.min_damage += item.min_damage;
        character.max_damage += item.max_damage;
        character.accuracy += item.accuracy;
        character.evasion += item.evade;
        character.armor += item.armor;
        character.adj_strength += item.strength as EOShort;
        character.adj_intelligence += item.intelligence as EOShort;
        character.adj_wisdom += item.wisdom as EOShort;
        character.adj_agility += item.agility as EOShort;
        character.adj_constitution += item.constitution as EOShort;
        character.adj_charisma += item.charisma as EOShort;
    }

    if character.weight > 250 {
        character.weight = 250;
    }

    let context = match context_map! {
        "base_str" => character.base_strength as i64,
        "base_int" => character.base_intelligence as i64,
        "base_wis" => character.base_wisdom as i64,
        "base_agi" => character.base_agility as i64,
        "base_con" => character.base_constitution as i64,
        "base_cha" => character.base_charisma as i64,
        "str" => character.adj_strength as i64,
        "int" => character.adj_intelligence as i64,
        "wis" => character.adj_wisdom as i64,
        "agi" => character.adj_agility as i64,
        "con" => character.adj_constitution as i64,
        "cha" => character.adj_charisma as i64,
        "level" => character.level as i64,
    } {
        Ok(context) => context,
        Err(e) => {
            error!("Failed to generate formula context: {}", e);
            return;
        },
    };

    character.max_hp += match eval_float_with_context(&FORMULAS.hp, &context) {
        Ok(max_hp) => max_hp.floor() as EOShort,
        Err(e) => {
            error!("Failed to calculate max_hp: {}", e);
            10
        },
    };

    character.max_tp += match eval_float_with_context(&FORMULAS.tp, &context) {
        Ok(max_tp) => max_tp.floor() as EOShort,
        Err(e) => {
            error!("Failed to calculate max_tp: {}", e);
            10
        },
    };

    character.max_sp += match eval_float_with_context(&FORMULAS.sp, &context) {
        Ok(max_sp) => max_sp.floor() as EOShort,
        Err(e) => {
            error!("Failed to calculate max_sp: {}", e);
            20
        },
    };

    character.max_weight = match eval_float_with_context(&FORMULAS.max_weight, &context) {
        Ok(max_weight) => max_weight.floor() as EOInt,
        Err(e) => {
            error!("Failed to calculate max_weight: {}", e);
            70
        },
    };

    let class_formulas = &FORMULAS.classes[class.class_type as usize];
    let damage = match eval_float_with_context(&class_formulas.damage, &context) {
        Ok(damage) => damage.floor() as EOShort,
        Err(e) => {
            error!("Failed to calculate damage: {}", e);
            1
        },
    };

    character.min_damage += damage;
    character.max_damage += damage;

    character.accuracy += match eval_float_with_context(&class_formulas.accuracy, &context) {
        Ok(accuracy) => accuracy.floor() as EOShort,
        Err(e) => {
            error!("Failed to calculate accuracy: {}", e);
            0
        },
    };

    character.armor += match eval_float_with_context(&class_formulas.defense, &context) {
        Ok(armor) => armor.floor() as EOShort,
        Err(e) => {
            error!("Failed to calculate armor: {}", e);
            0
        },
    };

    character.evasion += match eval_float_with_context(&class_formulas.evade, &context) {
        Ok(evasion) => evasion.floor() as EOShort,
        Err(e) => {
            error!("Failed to calculate evasion: {}", e);
            0
        },
    };

    if character.min_damage == 0 {
        character.min_damage = 1;
    }

    if character.max_damage == 0 {
        character.max_damage = 1;
    }
}