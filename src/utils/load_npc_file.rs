use eolib::{
    data::{decode_number, encode_number, EoReader, EoSerialize, EoWriter},
    protocol::r#pub::{Element, Enf, EnfRecord, NpcType},
};
use glob::glob;
use serde_json::Value;
use std::{fs::File, io::Read};

use bytes::Bytes;
use crc::{Crc, CRC_32_CKSUM};

use crate::SETTINGS;

use super::save_pub_file;

pub const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_CKSUM);

pub fn load_npc_file() -> Result<Enf, Box<dyn std::error::Error>> {
    if SETTINGS.server.generate_pub {
        load_json()
    } else {
        load_pub()
    }
}

fn load_json() -> Result<Enf, Box<dyn std::error::Error>> {
    let mut enf_file = Enf::default();

    for entry in glob("data/pub/npcs/*.json")? {
        let path = entry?;
        let mut file = File::open(path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;

        let v: Value = serde_json::from_str(&json)?;
        let record = EnfRecord {
            name: v["name"].as_str().unwrap_or_default().to_string(),
            graphic_id: v["graphicId"].as_u64().unwrap_or(0) as i32,
            race: v["race"].as_u64().unwrap_or(0) as i32,
            boss: v["boss"].as_u64().unwrap_or(0) as i32 == 1,
            child: v["child"].as_u64().unwrap_or(0) as i32 == 1,
            r#type: NpcType::from(v["type"].as_u64().unwrap_or(0) as i32),
            behavior_id: v["behaviorId"].as_u64().unwrap_or(0) as i32,
            hp: v["hp"].as_u64().unwrap_or(0) as i32,
            tp: v["tp"].as_u64().unwrap_or(0) as i32,
            min_damage: v["minDamage"].as_u64().unwrap_or(0) as i32,
            max_damage: v["maxDamage"].as_u64().unwrap_or(0) as i32,
            accuracy: v["accuracy"].as_u64().unwrap_or(0) as i32,
            evade: v["evade"].as_u64().unwrap_or(0) as i32,
            armor: v["armor"].as_u64().unwrap_or(0) as i32,
            return_damage: v["returnDamage"].as_u64().unwrap_or(0) as i32,
            element: Element::from(v["element"].as_u64().unwrap_or(0) as i32),
            element_damage: v["elementDamage"].as_u64().unwrap_or(0) as i32,
            element_weakness: Element::from(v["elementWeakness"].as_u64().unwrap_or(0) as i32),
            element_weakness_damage: v["elementWeaknessDamage"].as_u64().unwrap_or(0) as i32,
            level: v["level"].as_u64().unwrap_or(0) as i32,
            experience: v["experience"].as_u64().unwrap_or(0) as i32,
        };
        enf_file.npcs.push(record);
    }

    enf_file.npcs.push(EnfRecord {
        name: "eof".to_string(),
        ..Default::default()
    });

    enf_file.total_npcs_count = enf_file.npcs.len() as i32;

    let mut writer = EoWriter::new();
    enf_file.serialize(&mut writer).unwrap();
    let buf = writer.to_byte_array();

    let mut digest = CRC32.digest();
    digest.update(&buf[7..]);

    let checksum = digest.finalize();

    let encoded = encode_number(checksum as i32).unwrap();

    enf_file.rid = [
        decode_number(&encoded[0..=1]) as i32,
        decode_number(&encoded[2..=3]) as i32,
    ];

    save_pub_file(&enf_file, "data/pub/dtn001.enf")?;

    Ok(enf_file)
}

fn load_pub() -> Result<Enf, Box<dyn std::error::Error>> {
    let mut file = File::open("data/pub/dtn001.enf")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let bytes = Bytes::from(buf);
    let reader = EoReader::new(bytes);
    Ok(Enf::deserialize(&reader)?)
}
