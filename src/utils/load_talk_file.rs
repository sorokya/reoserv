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
    for entry in glob("data/pub/npcs/*.json")? {
        let path = entry?;
        let mut file = File::open(path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;

        let v: Value = serde_json::from_str(&json)?;

        let messages = v["talk_messages"].as_array().unwrap();
        if !messages.is_empty() {
            talk_file.npcs.push(TalkRecord {
                npc_id,
                rate: v["talk_rate"].as_u64().unwrap_or(0) as i32,
                messages: messages
                    .iter()
                    .map(|v| TalkMessageRecord {
                        message: v["message"].as_str().unwrap_or_default().to_string(),
                    })
                    .collect(),
            });
        }

        npc_id += 1;
    }

    save_pub_file(&talk_file, "data/pub/ttd001.etf")?;

    Ok(talk_file)
}

fn load_pub() -> Result<TalkFile, Box<dyn std::error::Error>> {
    if let Ok(mut file) = File::open("data/pub/serv_chats.epf") {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let bytes = Bytes::from(buf);
        let reader = EoReader::new(bytes);

        if reader.get_fixed_string(3) != "ETF" {
            return Err("Invalid file".into());
        }

        reader.get_short();
        reader.get_short();

        let mut etf = TalkFile::default();

        let num_records = reader.get_short();

        etf.npcs = Vec::with_capacity(num_records as usize);

        reader.get_char();

        for _ in 0..num_records {
            let mut record = TalkRecord {
                npc_id: reader.get_short(),
                ..Default::default()
            };

            reader.get_char();
            record.rate = reader.get_char();

            let num_messages = reader.get_char();
            record.messages = Vec::with_capacity(num_messages as usize);
            for _ in 0..num_messages {
                let length = reader.get_char();
                record.messages.push(TalkMessageRecord {
                    message: reader.get_fixed_string(length as usize),
                });
            }

            etf.npcs.push(record);
        }

        return Ok(etf);
    }

    let mut file = File::open("data/pub/ttd001.etf")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let bytes = Bytes::from(buf);
    let reader = EoReader::new(bytes);
    Ok(TalkFile::deserialize(&reader)?)
}
