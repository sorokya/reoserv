use std::{fs::File, io::Read};

use bytes::Bytes;
use eo::{
    data::{EOChar, EOShort, Serializeable, StreamReader},
    pubs::{TalkFile, TalkNpc},
};
use glob::glob;
use serde_json::Value;

use crate::SETTINGS;

pub fn load_talk_file() -> Result<TalkFile, Box<dyn std::error::Error>> {
    if SETTINGS.server.generate_pub {
        load_json()
    } else {
        load_pub()
    }
}

fn load_json() -> Result<TalkFile, Box<dyn std::error::Error>> {
    let mut talk_file = TalkFile::default();
    talk_file.magic = "ETF".to_string();

    for entry in glob("pub/npcs/*.json")? {
        let path = entry?;
        let mut file = File::open(path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;

        let v: Value = serde_json::from_str(&json)?;

        let npc_id = v["id"].as_u64().unwrap_or(0) as EOShort;
        let messages = v["talkMessages"].as_array().unwrap();
        if messages.len() > 0 {
            talk_file.npcs.push(TalkNpc {
                npc_id,
                rate: v["talkRate"].as_u64().unwrap_or(0) as EOChar,
                num_messages: messages.len() as EOChar,
                messages: messages
                    .iter()
                    .map(|v| v.as_str().unwrap_or_default().to_string())
                    .collect(),
            });
        }
    }

    Ok(talk_file)
}

fn load_pub() -> Result<TalkFile, Box<dyn std::error::Error>> {
    let mut file = File::open("pub/ttd001.etf")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let bytes = Bytes::from(buf);
    let reader = StreamReader::new(bytes);

    let mut talk_file = TalkFile::default();
    talk_file.deserialize(&reader);

    Ok(talk_file)
}
