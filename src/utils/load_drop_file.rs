use std::{fs::File, io::Read};

use bytes::Bytes;
use eolib::{
    data::{EoReader, EoSerialize},
    protocol::r#pub::server::{DropFile, DropNpcRecord, DropRecord},
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
    for entry in glob("data/pub/npcs/*.json")? {
        let path = entry?;
        let mut file = File::open(path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;

        let v: Value = serde_json::from_str(&json)?;

        let drops = v["drops"].as_array().unwrap();
        if !drops.is_empty() {
            drop_file.npcs.push(DropNpcRecord {
                npc_id,
                drops: drops
                    .iter()
                    .map(|v| DropRecord {
                        item_id: v["itemId"].as_u64().unwrap_or(0) as i32,
                        min_amount: v["min"].as_u64().unwrap_or(0) as i32,
                        max_amount: v["max"].as_u64().unwrap_or(0) as i32,
                        rate: v["rate"].as_u64().unwrap_or(0) as i32,
                    })
                    .collect(),
            });
        }
        npc_id += 1;
    }

    save_pub_file(&drop_file, "data/pub/dtd001.edf")?;

    Ok(drop_file)
}

fn load_pub() -> Result<DropFile, Box<dyn std::error::Error>> {
    if let Ok(mut file) = File::open("data/pub/serv_drops.epf") {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let bytes = Bytes::from(buf);
        let reader = EoReader::new(bytes);

        if reader.get_fixed_string(3) != "EDF" {
            return Err("Invalid file".into());
        }

        reader.get_short();
        reader.get_short();

        let mut edf = DropFile::default();

        let num_records = reader.get_short();

        edf.npcs = Vec::with_capacity(num_records as usize);

        reader.get_char();

        for _ in 0..num_records {
            let mut record = DropNpcRecord {
                npc_id: reader.get_short(),
                ..Default::default()
            };

            let num_drops = reader.get_short();

            record.drops = Vec::with_capacity(num_drops as usize);
            for _ in 0..num_drops {
                let item_id = reader.get_short();
                let min_amount = reader.get_three();
                let max_amount = reader.get_three();
                let rate = ((reader.get_short() as f32) / 10_000. * 64_000.).floor() as i32;
                record.drops.push(DropRecord {
                    item_id,
                    min_amount,
                    max_amount,
                    rate,
                });
            }

            edf.npcs.push(record);
        }

        return Ok(edf);
    }

    if let Ok(mut file) = File::open("data/pub/dtd001.edf") {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let bytes = Bytes::from(buf);
        let reader = EoReader::new(bytes);
        return Ok(DropFile::deserialize(&reader)?);
    }

    Ok(DropFile::default())
}
