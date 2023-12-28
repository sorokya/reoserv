use glob::glob;
use serde_json::Value;
use std::{fs::File, io::Read};

use bytes::Bytes;
use crc::{Crc, CRC_32_CKSUM};
use eo::{
    data::{
        decode_number, encode_number, i32, EOInt, i32, Serializeable, StreamBuilder,
        StreamReader,
    },
    pubs::{EnfFile, EnfNpc, EnfNpcType},
};

use crate::SETTINGS;

use super::save_pub_file;

pub const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_CKSUM);

pub fn load_npc_file() -> Result<EnfFile, Box<dyn std::error::Error>> {
    if SETTINGS.server.generate_pub {
        load_json()
    } else {
        load_pub()
    }
}

fn load_json() -> Result<EnfFile, Box<dyn std::error::Error>> {
    let mut enf_file = EnfFile::default();
    enf_file.magic = "ENF".to_string();

    for entry in glob("pub/npcs/*.json")? {
        let path = entry?;
        let mut file = File::open(path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;

        let v: Value = serde_json::from_str(&json)?;
        let record = EnfNpc {
            name: v["name"].as_str().unwrap_or_default().to_string(),
            graphic_id: v["graphicId"].as_u64().unwrap_or(0) as i32,
            race: v["race"].as_u64().unwrap_or(0) as i32,
            boss: v["boss"].as_u64().unwrap_or(0) as i32,
            child: v["child"].as_u64().unwrap_or(0) as i32,
            r#type: EnfNpcType::from_short(v["type"].as_u64().unwrap_or(0) as i32)
                .unwrap_or_default(),
            behavior_id: v["behaviorId"].as_u64().unwrap_or(0) as i32,
            hp: v["hp"].as_u64().unwrap_or(0) as EOInt,
            tp: v["tp"].as_u64().unwrap_or(0) as i32,
            min_damage: v["minDamage"].as_u64().unwrap_or(0) as i32,
            max_damage: v["maxDamage"].as_u64().unwrap_or(0) as i32,
            accuracy: v["accuracy"].as_u64().unwrap_or(0) as i32,
            evade: v["evade"].as_u64().unwrap_or(0) as i32,
            armor: v["armor"].as_u64().unwrap_or(0) as i32,
            return_damage: v["returnDamage"].as_u64().unwrap_or(0) as i32,
            element: v["element"].as_u64().unwrap_or(0) as i32,
            element_damage: v["elementDamage"].as_u64().unwrap_or(0) as i32,
            element_weakness: v["elementWeakness"].as_u64().unwrap_or(0) as i32,
            element_weakness_damage: v["elementWeaknessDamage"].as_u64().unwrap_or(0) as i32,
            level: v["level"].as_u64().unwrap_or(0) as i32,
            experience: v["experience"].as_u64().unwrap_or(0) as EOInt,
        };
        enf_file.npcs.push(record);
        enf_file.num_npcs += 1;
    }

    enf_file.npcs.push(EnfNpc {
        name: "eof".to_string(),
        ..Default::default()
    });
    enf_file.num_npcs += 1;

    let mut builder = StreamBuilder::new();
    enf_file.serialize(&mut builder);
    let buf = builder.get();

    let mut digest = CRC32.digest();
    digest.update(&buf[7..]);

    let checksum = digest.finalize();

    let encoded = encode_number(checksum);

    enf_file.rid = [
        decode_number(&encoded[0..=1]) as i32,
        decode_number(&encoded[2..=3]) as i32,
    ];

    save_pub_file(&enf_file, "pub/dtn001.enf")?;

    Ok(enf_file)
}

fn load_pub() -> Result<EnfFile, Box<dyn std::error::Error>> {
    let mut file = File::open("pub/dtn001.enf")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let bytes = Bytes::from(buf);
    let reader = StreamReader::new(bytes);

    let mut npc_file = EnfFile::default();
    npc_file.deserialize(&reader);
    Ok(npc_file)
}
