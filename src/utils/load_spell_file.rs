use crc::{Crc, CRC_32_CKSUM};
use eolib::{
    data::{decode_number, encode_number, EoReader, EoSerialize, EoWriter},
    protocol::r#pub::{
        Element, Esf, EsfRecord, SkillNature, SkillTargetRestrict, SkillTargetType, SkillType,
    },
};
use glob::glob;
use serde_json::Value;

use std::{fs::File, io::Read};

use bytes::Bytes;

use crate::SETTINGS;

use super::save_pub_file;

pub const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_CKSUM);

pub fn load_spell_file() -> Result<Esf, Box<dyn std::error::Error>> {
    if SETTINGS.server.generate_pub {
        load_json()
    } else {
        load_pub()
    }
}

fn load_json() -> Result<Esf, Box<dyn std::error::Error>> {
    let mut esf_file = Esf::default();

    for entry in glob("data/pub/spells/*.json")? {
        let path = entry?;
        let mut file = File::open(path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;

        let v: Value = serde_json::from_str(&json)?;
        let record = EsfRecord {
            name: v["name"].as_str().unwrap_or_default().to_string(),
            chant: v["chant"].as_str().unwrap_or_default().to_string(),
            icon_id: v["icon_id"].as_u64().unwrap_or(0) as i32,
            graphic_id: v["graphic_id"].as_u64().unwrap_or(0) as i32,
            tp_cost: v["tp_cost"].as_u64().unwrap_or(0) as i32,
            sp_cost: v["sp_cost"].as_u64().unwrap_or(0) as i32,
            cast_time: v["cast_time"].as_u64().unwrap_or(0) as i32,
            nature: SkillNature::from(v["nature"].as_u64().unwrap_or(0) as i32),
            r#type: SkillType::from(v["type"].as_u64().unwrap_or(0) as i32),
            element: Element::from(v["element"].as_u64().unwrap_or(0) as i32),
            element_power: v["element_power"].as_u64().unwrap_or(0) as i32,
            target_restrict: SkillTargetRestrict::from(
                v["target_restrict"].as_u64().unwrap_or(0) as i32
            ),
            target_type: SkillTargetType::from(v["target_type"].as_u64().unwrap_or(0) as i32),
            target_time: v["target_time"].as_u64().unwrap_or(0) as i32,
            max_skill_level: v["max_skill_level"].as_u64().unwrap_or(0) as i32,
            min_damage: v["min_damage"].as_u64().unwrap_or(0) as i32,
            max_damage: v["max_damage"].as_u64().unwrap_or(0) as i32,
            accuracy: v["accuracy"].as_u64().unwrap_or(0) as i32,
            evade: v["evade"].as_u64().unwrap_or(0) as i32,
            armor: v["armor"].as_u64().unwrap_or(0) as i32,
            return_damage: v["return_damage"].as_u64().unwrap_or(0) as i32,
            hp_heal: v["heal_hp"].as_u64().unwrap_or(0) as i32,
            tp_heal: v["heal_tp"].as_u64().unwrap_or(0) as i32,
            sp_heal: v["heal_sp"].as_u64().unwrap_or(0) as i32,
            str: v["str"].as_u64().unwrap_or(0) as i32,
            intl: v["intl"].as_u64().unwrap_or(0) as i32,
            wis: v["wis"].as_u64().unwrap_or(0) as i32,
            agi: v["agi"].as_u64().unwrap_or(0) as i32,
            con: v["con"].as_u64().unwrap_or(0) as i32,
            cha: v["cha"].as_u64().unwrap_or(0) as i32,
        };
        esf_file.skills.push(record);
    }

    esf_file.skills.push(EsfRecord {
        name: "eof".to_string(),
        ..Default::default()
    });

    esf_file.total_skills_count = esf_file.skills.len() as i32;

    let mut writer = EoWriter::new();
    esf_file.serialize(&mut writer).unwrap();
    let buf = writer.to_byte_array();

    let mut digest = CRC32.digest();
    digest.update(&buf[7..]);

    let checksum = digest.finalize();

    let encoded = encode_number(checksum as i32).unwrap();

    esf_file.rid = [
        decode_number(&encoded[0..=1]) as i32,
        decode_number(&encoded[2..=3]) as i32,
    ];

    save_pub_file(&esf_file, "data/pub/dsl001.esf")?;

    Ok(esf_file)
}

fn load_pub() -> Result<Esf, Box<dyn std::error::Error>> {
    if let Ok(mut file) = File::open("data/pub/dsl001.esf") {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let bytes = Bytes::from(buf);
        let reader = EoReader::new(bytes);
        return Ok(Esf::deserialize(&reader)?);
    }

    Ok(Esf::default())
}
