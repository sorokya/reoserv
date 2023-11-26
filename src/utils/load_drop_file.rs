use std::{fs::File, io::Read};

use bytes::Bytes;
use eo::{
    data::{EOInt, EOShort, Serializeable, StreamReader},
    pubs::{Drop, DropFile, DropNpc},
};
use glob::glob;
use serde_json::Value;

use crate::SETTINGS;

use super::save_pub_file;

pub fn load_drop_file() -> Result<DropFile, Box<dyn std::error::Error>> {
    if SETTINGS.server.generate_pub {
        load_json()
    } else {
        load_pub()
    }
}

fn load_json() -> Result<DropFile, Box<dyn std::error::Error>> {
    let mut drop_file = DropFile::default();
    drop_file.magic = "EDF".to_string();

    for entry in glob("pub/npcs/*.json")? {
        let path = entry?;
        let mut file = File::open(path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;

        let v: Value = serde_json::from_str(&json)?;

        let npc_id = v["id"].as_u64().unwrap_or(0) as EOShort;
        let drops = v["drops"].as_array().unwrap();
        if drops.len() > 0 {
            drop_file.npcs.push(DropNpc {
                npc_id,
                num_of_drops: drops.len() as EOShort,
                drops: drops
                    .iter()
                    .map(|v| Drop {
                        item_id: v["itemId"].as_u64().unwrap_or(0) as EOShort,
                        min: v["min"].as_u64().unwrap_or(0) as EOInt,
                        max: v["max"].as_u64().unwrap_or(0) as EOInt,
                        rate: v["rate"].as_u64().unwrap_or(0) as EOShort,
                    })
                    .collect(),
            });
        }
    }

    save_pub_file(&drop_file, "pub/dtd001.edf")?;

    Ok(drop_file)
}

fn load_pub() -> Result<DropFile, Box<dyn std::error::Error>> {
    let mut file = File::open("pub/dtd001.edf")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let bytes = Bytes::from(buf);
    let reader = StreamReader::new(bytes);

    let mut drop_file = DropFile::default();
    drop_file.deserialize(&reader);
    Ok(drop_file)
}
