use std::{fs::File, io::Read};

use bytes::Bytes;
use eo::{
    data::{EOChar, EOShort, Serializeable, StreamReader},
    pubs::{Inn, InnFile},
};
use glob::glob;
use serde_json::Value;

use crate::SETTINGS;

pub fn load_inn_file() -> Result<InnFile, Box<dyn std::error::Error>> {
    if SETTINGS.server.generate_pub {
        load_json()
    } else {
        load_pub()
    }
}

fn load_json() -> Result<InnFile, Box<dyn std::error::Error>> {
    let mut inn_file = InnFile::default();
    inn_file.magic = "EID".to_string();

    for entry in glob("pub/inns/*.json")? {
        let path = entry?;
        let mut file = File::open(path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;

        let v: Value = serde_json::from_str(&json)?;

        inn_file.inns.push(Inn {
            vendor_id: v["behaviorId"].as_u64().unwrap_or(0) as EOShort,
            name: v["name"].as_str().unwrap_or_default().to_string(),
            spawn_map: v["spawnMap"].as_u64().unwrap_or(0) as EOShort,
            spawn_x: v["spawnX"].as_u64().unwrap_or(0) as EOChar,
            spawn_y: v["spawnY"].as_u64().unwrap_or(0) as EOChar,
            sleep_map: v["sleepMap"].as_u64().unwrap_or(0) as EOShort,
            sleep_x: v["sleepX"].as_u64().unwrap_or(0) as EOChar,
            sleep_y: v["sleepY"].as_u64().unwrap_or(0) as EOChar,
            alt_spawn_enabled: v["altSpawnEnabled"].as_u64().unwrap_or(0) as EOChar,
            alt_spawn_map: v["altSpawnMap"].as_u64().unwrap_or(0) as EOShort,
            alt_spawn_x: v["altSpawnX"].as_u64().unwrap_or(0) as EOChar,
            alt_spawn_y: v["altSpawnY"].as_u64().unwrap_or(0) as EOChar,
            question1: v["question1"].as_str().unwrap_or_default().to_string(),
            answer1: v["answer1"].as_str().unwrap_or_default().to_string(),
            question2: v["question2"].as_str().unwrap_or_default().to_string(),
            answer2: v["answer2"].as_str().unwrap_or_default().to_string(),
            question3: v["question3"].as_str().unwrap_or_default().to_string(),
            answer3: v["answer3"].as_str().unwrap_or_default().to_string(),
        });
    }

    Ok(inn_file)
}

fn load_pub() -> Result<InnFile, Box<dyn std::error::Error>> {
    let mut file = File::open("pub/din001.eid")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let bytes = Bytes::from(buf);
    let reader = StreamReader::new(bytes);

    let mut inn_file = InnFile::default();
    inn_file.deserialize(&reader);
    Ok(inn_file)
}
