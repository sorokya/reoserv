use std::{cmp, fs::File, io::Read};

use bytes::Bytes;
use eolib::{
    data::{EoReader, EoSerialize},
    protocol::r#pub::server::{InnFile, InnQuestionRecord, InnRecord},
};
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

    for entry in glob("data/pub/inns/*.json")? {
        let path = entry?;
        let mut file = File::open(path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;

        let v: Value = serde_json::from_str(&json)?;

        let mut questions = Vec::with_capacity(3);

        for i in 1..=3 {
            questions.push(InnQuestionRecord {
                question: v[&format!("question{}", i)]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                answer: v[&format!("answer{}", i)]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
            });
        }

        if questions.len() != 3 {
            panic!(
                "Inn {} has {} questions, but should have 3",
                v["name"].as_str().unwrap_or_default(),
                questions.len()
            );
        }

        inn_file.inns.push(InnRecord {
            behavior_id: v["behaviorId"].as_u64().unwrap_or(0) as i32,
            name: v["name"].as_str().unwrap_or_default().to_string(),
            spawn_map: v["spawnMap"].as_u64().unwrap_or(0) as i32,
            spawn_x: v["spawnX"].as_u64().unwrap_or(0) as i32,
            spawn_y: v["spawnY"].as_u64().unwrap_or(0) as i32,
            sleep_map: v["sleepMap"].as_u64().unwrap_or(0) as i32,
            sleep_x: v["sleepX"].as_u64().unwrap_or(0) as i32,
            sleep_y: v["sleepY"].as_u64().unwrap_or(0) as i32,
            alternate_spawn_enabled: v["altSpawnEnabled"].as_u64().unwrap_or(0) == 1,
            alternate_spawn_map: v["altSpawnMap"].as_u64().unwrap_or(0) as i32,
            alternate_spawn_x: v["altSpawnX"].as_u64().unwrap_or(0) as i32,
            alternate_spawn_y: v["altSpawnY"].as_u64().unwrap_or(0) as i32,
            questions: [
                questions[0].clone(),
                questions[1].clone(),
                questions[2].clone(),
            ],
        });
    }

    save_pub_file(&inn_file, "data/pub/din001.eid")?;

    Ok(inn_file)
}

fn load_pub() -> Result<InnFile, Box<dyn std::error::Error>> {
    if let Ok(mut file) = File::open("data/pub/serv_inns.epf") {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let bytes = Bytes::from(buf);
        let reader = EoReader::new(bytes);
        if reader.get_fixed_string(3) != "EID" {
            return Err("Invalid file".into());
        }

        reader.get_short();
        reader.get_short();

        let mut eif = InnFile::default();

        let num_records = reader.get_short();

        eif.inns = Vec::with_capacity(num_records as usize);

        reader.get_char();

        for _ in 0..num_records {
            let mut record = InnRecord::default();
            record.behavior_id = reader.get_short();
            let name_length = reader.get_char();
            record.name = reader.get_fixed_string(name_length as usize);

            // sleep cost
            reader.get_three();

            record.sleep_map = reader.get_short();
            record.sleep_x = reader.get_char();
            record.sleep_y = reader.get_char();
            record.spawn_map = reader.get_short();
            record.spawn_x = reader.get_char();
            record.spawn_y = reader.get_char();
            record.alternate_spawn_enabled = reader.get_short() != 0;
            record.alternate_spawn_map = reader.get_short();
            record.alternate_spawn_x = reader.get_char();
            record.alternate_spawn_y = reader.get_char();

            let num_questions = reader.get_char();
            for i in 0..cmp::min(num_questions as usize, 3) {
                let question_length = reader.get_char();
                let question = reader.get_fixed_string(question_length as usize);
                let answer_length = reader.get_char();
                let answer = reader.get_fixed_string(answer_length as usize);
                record.questions[i] = InnQuestionRecord { question, answer };
            }
            eif.inns.push(record);
        }

        return Ok(eif);
    }

    if let Ok(mut file) = File::open("data/pub/din001.eid") {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let bytes = Bytes::from(buf);
        let reader = EoReader::new(bytes);
        return Ok(InnFile::deserialize(&reader)?);
    }

    Ok(InnFile::default())
}
