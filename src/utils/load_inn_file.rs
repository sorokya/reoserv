use std::{fs::File, io::Read};

use bytes::Bytes;
use eolib::{protocol::r#pub::server::{InnRecord, InnFile}, data::{EoReader, EoSerialize}};
use glob::glob;
use serde_json::Value;

use crate::SETTINGS;

use super::save_pub_file;

pub fn load_inn_file() -> Result<InnFile, Box<dyn std::error::Error>> {
    if SETTINGS.server.generate_pub {
        load_json()
    } else {
        load_pub()
    }
}

fn load_json() -> Result<InnFile, Box<dyn std::error::Error>> {
    let mut inn_file = InnFile::default();

    for entry in glob("pub/inns/*.json")? {
        let path = entry?;
        let mut file = File::open(path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;

        let v: Value = serde_json::from_str(&json)?;

        inn_file.inns.push(InnRecord {
            behavior_id: v["behaviorId"].as_u64().unwrap_or(0) as i32,
            name: v["name"].as_str().unwrap_or_default().to_string(),
            spawn_map: v["spawnMap"].as_u64().unwrap_or(0) as i32,
            spawn_x: v["spawnX"].as_u64().unwrap_or(0) as i32,
            spawn_y: v["spawnY"].as_u64().unwrap_or(0) as i32,
            sleep_map: v["sleepMap"].as_u64().unwrap_or(0) as i32,
            sleep_x: v["sleepX"].as_u64().unwrap_or(0) as i32,
            sleep_y: v["sleepY"].as_u64().unwrap_or(0) as i32,
            alt_spawn_enabled: v["altSpawnEnabled"].as_u64().unwrap_or(0) == 1,
            alt_spawn_map: v["altSpawnMap"].as_u64().unwrap_or(0) as i32,
            alt_spawn_x: v["altSpawnX"].as_u64().unwrap_or(0) as i32,
            alt_spawn_y: v["altSpawnY"].as_u64().unwrap_or(0) as i32,
            question1: v["question1"].as_str().unwrap_or_default().to_string(),
            answer1: v["answer1"].as_str().unwrap_or_default().to_string(),
            question2: v["question2"].as_str().unwrap_or_default().to_string(),
            answer2: v["answer2"].as_str().unwrap_or_default().to_string(),
            question3: v["question3"].as_str().unwrap_or_default().to_string(),
            answer3: v["answer3"].as_str().unwrap_or_default().to_string(),
        });
    }

    save_pub_file(&inn_file, "pub/din001.eid")?;

    Ok(inn_file)
}

fn load_pub() -> Result<InnFile, Box<dyn std::error::Error>> {
    let mut file = File::open("pub/din001.eid")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let bytes = Bytes::from(buf);
    let reader = EoReader::new(bytes);
    Ok(InnFile::deserialize(&reader)?)
}
