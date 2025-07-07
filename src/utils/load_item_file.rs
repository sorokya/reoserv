use std::{fs::File, io::Read};

use bytes::Bytes;
use crc::{Crc, CRC_32_CKSUM};

use eolib::{
    data::{decode_number, encode_number, EoReader, EoSerialize, EoWriter},
    protocol::r#pub::{Eif, EifRecord, Element, ItemSize, ItemSpecial, ItemSubtype, ItemType},
};
use glob::glob;
use serde_json::Value;

use crate::SETTINGS;

use super::save_pub_file;

pub const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_CKSUM);

pub fn load_item_file() -> Result<Eif, Box<dyn std::error::Error>> {
    if SETTINGS.server.generate_pub {
        load_json()
    } else {
        load_pub()
    }
}

fn load_json() -> Result<Eif, Box<dyn std::error::Error>> {
    let mut eif_file = Eif::default();

    for entry in glob("data/pub/items/*.json")? {
        let path = entry?;
        let mut file = File::open(path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;

        let v: Value = serde_json::from_str(&json)?;
        let record = EifRecord {
            name: v["name"].as_str().unwrap_or_default().to_string(),
            graphic_id: v["graphic_id"].as_u64().unwrap_or(0) as i32,
            r#type: ItemType::from(v["type"].as_u64().unwrap_or(0) as i32),
            subtype: ItemSubtype::from(v["subtype"].as_u64().unwrap_or(0) as i32),
            special: ItemSpecial::from(v["special"].as_u64().unwrap_or(0) as i32),
            hp: v["hp"].as_u64().unwrap_or(0) as i32,
            tp: v["tp"].as_u64().unwrap_or(0) as i32,
            min_damage: v["min_damage"].as_u64().unwrap_or(0) as i32,
            max_damage: v["max_damage"].as_u64().unwrap_or(0) as i32,
            accuracy: v["accuracy"].as_u64().unwrap_or(0) as i32,
            evade: v["evade"].as_u64().unwrap_or(0) as i32,
            armor: v["armor"].as_u64().unwrap_or(0) as i32,
            return_damage: v["return_damage"].as_u64().unwrap_or(0) as i32,
            str: v["str"].as_u64().unwrap_or(0) as i32,
            intl: v["intl"].as_u64().unwrap_or(0) as i32,
            wis: v["wis"].as_u64().unwrap_or(0) as i32,
            agi: v["agi"].as_u64().unwrap_or(0) as i32,
            con: v["con"].as_u64().unwrap_or(0) as i32,
            cha: v["cha"].as_u64().unwrap_or(0) as i32,
            light_resistance: v["light_resistance"].as_u64().unwrap_or(0) as i32,
            dark_resistance: v["dark_resistance"].as_u64().unwrap_or(0) as i32,
            earth_resistance: v["earth_resistance"].as_u64().unwrap_or(0) as i32,
            air_resistance: v["air_resistance"].as_u64().unwrap_or(0) as i32,
            water_resistance: v["water_resistance"].as_u64().unwrap_or(0) as i32,
            fire_resistance: v["fire_resistance"].as_u64().unwrap_or(0) as i32,
            spec1: v["spec1"].as_u64().unwrap_or(0) as i32,
            spec2: v["spec2"].as_u64().unwrap_or(0) as i32,
            spec3: v["spec3"].as_u64().unwrap_or(0) as i32,
            level_requirement: v["level_requirement"].as_u64().unwrap_or(0) as i32,
            class_requirement: v["class_requirement"].as_u64().unwrap_or(0) as i32,
            str_requirement: v["str_requirement"].as_u64().unwrap_or(0) as i32,
            int_requirement: v["int_requirement"].as_u64().unwrap_or(0) as i32,
            wis_requirement: v["wis_requirement"].as_u64().unwrap_or(0) as i32,
            agi_requirement: v["agi_requirement"].as_u64().unwrap_or(0) as i32,
            con_requirement: v["con_requirement"].as_u64().unwrap_or(0) as i32,
            cha_requirement: v["cha_requirement"].as_u64().unwrap_or(0) as i32,
            element: Element::from(v["element"].as_u64().unwrap_or(0) as i32),
            element_damage: v["element_damage"].as_u64().unwrap_or(0) as i32,
            weight: v["weight"].as_u64().unwrap_or(0) as i32,
            size: ItemSize::from(v["size"].as_u64().unwrap_or(0) as i32),
        };
        eif_file.items.push(record);
    }

    eif_file.items.push(EifRecord {
        name: "eof".to_string(),
        ..Default::default()
    });

    eif_file.total_items_count = eif_file.items.len() as i32;

    let mut writer = EoWriter::new();
    eif_file.serialize(&mut writer).unwrap();
    let buf = writer.to_byte_array();

    let mut digest = CRC32.digest();
    digest.update(&buf[7..]);

    let checksum = digest.finalize();

    let encoded = encode_number(checksum as i32).unwrap();

    eif_file.rid = [
        decode_number(&encoded[0..=1]) as i32,
        decode_number(&encoded[2..=3]) as i32,
    ];

    save_pub_file(&eif_file, "data/pub/dat001.eif")?;

    Ok(eif_file)
}

fn load_pub() -> Result<Eif, Box<dyn std::error::Error>> {
    if let Ok(mut file) = File::open("data/pub/dat001.eif") {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let bytes = Bytes::from(buf);
        let reader = EoReader::new(bytes);
        return Ok(Eif::deserialize(&reader)?);
    }

    Ok(Eif::default())
}
