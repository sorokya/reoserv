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

    for entry in glob("pub/skill_masters/*.json")? {
        let path = entry?;
        let mut file = File::open(path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;

        let v: Value = serde_json::from_str(&json)?;

        let skills = v["skills"].as_array().unwrap();

        skill_master_file.skill_masters.push(SkillMasterRecord {
            behavior_id: v["behaviorId"].as_u64().unwrap_or(0) as i32,
            name: v["name"].as_str().unwrap_or_default().to_string(),
            min_level: v["minLevel"].as_u64().unwrap_or(0) as i32,
            max_level: v["maxLevel"].as_u64().unwrap_or(0) as i32,
            class_requirement: v["classRequirement"].as_u64().unwrap_or(0) as i32,
            skills: skills
                .iter()
                .map(|v| {
                    let skill_requirements: Vec<i32> = v["skill_requirements"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_u64().unwrap_or_default() as i32)
                        .collect();

                    SkillMasterSkillRecord {
                        skill_id: v["id"].as_u64().unwrap_or(0) as i32,
                        level_requirement: v["levelRequirement"].as_u64().unwrap_or(0) as i32,
                        class_requirement: v["classRequirement"].as_u64().unwrap_or(0) as i32,
                        price: v["price"].as_u64().unwrap_or(0) as i32,
                        skill_requirements: [
                            skill_requirements[0],
                            skill_requirements[1],
                            skill_requirements[2],
                            skill_requirements[3],
                        ],
                        str_requirement: v["strRequirement"].as_u64().unwrap_or(0) as i32,
                        int_requirement: v["intRequirement"].as_u64().unwrap_or(0) as i32,
                        wis_requirement: v["wisRequirement"].as_u64().unwrap_or(0) as i32,
                        agi_requirement: v["agiRequirement"].as_u64().unwrap_or(0) as i32,
                        con_requirement: v["conRequirement"].as_u64().unwrap_or(0) as i32,
                        cha_requirement: v["chaRequirement"].as_u64().unwrap_or(0) as i32,
                    }
                })
                .collect(),
        });
    }

    save_pub_file(&skill_master_file, "pub/dsm001.emf")?;

    Ok(skill_master_file)
}

fn load_pub() -> Result<SkillMasterFile, Box<dyn std::error::Error>> {
    let mut file = File::open("pub/dsm001.emf")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let bytes = Bytes::from(buf);
    let reader = EoReader::new(bytes);
    Ok(SkillMasterFile::deserialize(&reader)?)
}
