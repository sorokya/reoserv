use std::{fs::File, io::Read};

use bytes::Bytes;
use eolib::{
    data::{EoReader, EoSerialize},
    protocol::r#pub::server::{Drop, DropFile, DropRecord},
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

    let mut npc_id = 1;
    for entry in glob("pub/npcs/*.json")? {
        let path = entry?;
        let mut file = File::open(path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;

        let v: Value = serde_json::from_str(&json)?;

        let drops = v["drops"].as_array().unwrap();
        if !drops.is_empty() {
            drop_file.npcs.push(DropRecord {
                npc_id,
                drops: drops
                    .iter()
                    .map(|v| Drop {
                        item_id: v["itemId"].as_u64().unwrap_or(0) as i32,
                        min_amount: v["minAmount"].as_u64().unwrap_or(0) as i32,
                        max_amount: v["maxAmount"].as_u64().unwrap_or(0) as i32,
                        rate: v["rate"].as_u64().unwrap_or(0) as i32,
                    })
                    .collect(),
            });
        }
        npc_id += 1;
    }

    save_pub_file(&drop_file, "pub/dtd001.edf")?;

    Ok(drop_file)
}

fn load_pub() -> Result<DropFile, Box<dyn std::error::Error>> {
    let mut file = File::open("pub/dtd001.edf")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let bytes = Bytes::from(buf);
    let reader = EoReader::new(bytes);
    Ok(DropFile::deserialize(&reader)?)
}
