use std::{fs::File, io::Read};

use bytes::Bytes;
use eolib::{
    data::{EoReader, EoSerialize},
    protocol::r#pub::server::{TalkFile, TalkMessageRecord, TalkRecord},
};
use glob::glob;
use serde_json::Value;

use crate::SETTINGS;

use super::save_pub_file;

pub fn load_talk_file() -> Result<TalkFile, Box<dyn std::error::Error>> {
    if SETTINGS.server.generate_pub {
        load_json()
    } else {
        load_pub()
    }
}

fn load_json() -> Result<TalkFile, Box<dyn std::error::Error>> {
    let mut talk_file = TalkFile::default();

    let mut npc_id = 1;
    for entry in glob("pub/npcs/*.json")? {
        let path = entry?;
        let mut file = File::open(path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;

        let v: Value = serde_json::from_str(&json)?;

        let messages = v["talkMessages"].as_array().unwrap();
        if !messages.is_empty() {
            talk_file.npcs.push(TalkRecord {
                npc_id,
                rate: v["talkRate"].as_u64().unwrap_or(0) as i32,
                messages: messages
                    .iter()
                    .map(|v| TalkMessageRecord {
                        message: v.as_str().unwrap_or_default().to_string(),
                    })
                    .collect(),
            });
        }

        npc_id += 1;
    }

    save_pub_file(&talk_file, "pub/ttd001.etf")?;

    Ok(talk_file)
}

fn load_pub() -> Result<TalkFile, Box<dyn std::error::Error>> {
    let mut file = File::open("pub/ttd001.etf")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let bytes = Bytes::from(buf);
    let reader = EoReader::new(bytes);
    Ok(TalkFile::deserialize(&reader)?)
}
