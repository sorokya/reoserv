use std::{fs::File, io::Read};

use bytes::Bytes;
use eolib::{
    data::{EoReader, EoSerialize},
    protocol::r#pub::server::{SkillMasterFile, SkillMasterRecord, SkillMasterSkillRecord},
};
use glob::glob;
use serde_json::Value;

use crate::SETTINGS;

use super::save_pub_file;

pub fn load_skill_master_file() -> Result<SkillMasterFile, Box<dyn std::error::Error>> {
    if SETTINGS.server.generate_pub {
        load_json()
    } else {
        load_pub()
    }
}

fn load_json() -> Result<SkillMasterFile, Box<dyn std::error::Error>> {
    let mut skill_master_file = SkillMasterFile::default();

    for entry in glob("data/pub/skill_masters/*.json")? {
        let path = entry?;
        let mut file = File::open(path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;

        let v: Value = serde_json::from_str(&json)?;

        let skills = v["skills"].as_array().unwrap();

        skill_master_file.skill_masters.push(SkillMasterRecord {
            behavior_id: v["behavior_id"].as_u64().unwrap_or(0) as i32,
            name: v["name"].as_str().unwrap_or_default().to_string(),
            min_level: v["min_level"].as_u64().unwrap_or(0) as i32,
            max_level: v["max_level"].as_u64().unwrap_or(0) as i32,
            class_requirement: v["class_requirement"].as_u64().unwrap_or(0) as i32,
            skills: skills
                .iter()
                .map(|v| {
                    let skill_requirements = v["skill_requirements"].as_array().unwrap();
                    SkillMasterSkillRecord {
                        skill_id: v["id"].as_u64().unwrap_or(0) as i32,
                        level_requirement: v["level_requirement"].as_u64().unwrap_or(0) as i32,
                        class_requirement: v["class_requirement"].as_u64().unwrap_or(0) as i32,
                        price: v["price"].as_u64().unwrap_or(0) as i32,
                        skill_requirements: skill_requirements
                            .iter()
                            .map(|v| v.as_u64().unwrap_or_default() as i32)
                            .collect::<Vec<_>>()
                            .try_into()
                            .unwrap(),
                        str_requirement: v["str_requirement"].as_u64().unwrap_or(0) as i32,
                        int_requirement: v["int_requirement"].as_u64().unwrap_or(0) as i32,
                        wis_requirement: v["wis_requirement"].as_u64().unwrap_or(0) as i32,
                        agi_requirement: v["agi_requirement"].as_u64().unwrap_or(0) as i32,
                        con_requirement: v["con_requirement"].as_u64().unwrap_or(0) as i32,
                        cha_requirement: v["cha_requirement"].as_u64().unwrap_or(0) as i32,
                    }
                })
                .collect(),
        });
    }

    save_pub_file(&skill_master_file, "data/pub/dsm001.emf")?;

    Ok(skill_master_file)
}

fn load_pub() -> Result<SkillMasterFile, Box<dyn std::error::Error>> {
    if let Ok(mut file) = File::open("data/pub/serv_trainers.epf") {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let bytes = Bytes::from(buf);
        let reader = EoReader::new(bytes);

        if reader.get_fixed_string(3) != "ETF" {
            return Err("Invalid file".into());
        }

        reader.get_short();
        reader.get_short();

        let mut emf = SkillMasterFile::default();

        let num_records = reader.get_short();

        emf.skill_masters = Vec::with_capacity(num_records as usize);

        reader.get_char();

        for _ in 0..num_records {
            let mut record = SkillMasterRecord {
                behavior_id: reader.get_short(),
                ..Default::default()
            };

            let name_length = reader.get_char();
            record.name = reader.get_fixed_string(name_length as usize);

            record.min_level = reader.get_short();
            record.max_level = reader.get_short();
            record.class_requirement = reader.get_char();

            let num_skills = reader.get_short();
            record.skills = Vec::with_capacity(num_skills as usize);

            for _ in 0..num_skills {
                record.skills.push(SkillMasterSkillRecord {
                    skill_id: reader.get_short(),
                    level_requirement: reader.get_short(),
                    class_requirement: reader.get_char(),
                    price: reader.get_three(),
                    skill_requirements: [
                        reader.get_short(),
                        reader.get_short(),
                        reader.get_short(),
                        reader.get_short(),
                    ],
                    str_requirement: reader.get_short(),
                    int_requirement: reader.get_short(),
                    wis_requirement: reader.get_short(),
                    agi_requirement: reader.get_short(),
                    con_requirement: reader.get_short(),
                    cha_requirement: reader.get_short(),
                });
            }

            emf.skill_masters.push(record);
        }

        return Ok(emf);
    }

    if let Ok(mut file) = File::open("data/pub/dsm001.emf") {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let bytes = Bytes::from(buf);
        let reader = EoReader::new(bytes);
        return Ok(SkillMasterFile::deserialize(&reader)?);
    }

    Ok(SkillMasterFile::default())
}
