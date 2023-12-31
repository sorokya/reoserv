use crc::{Crc, CRC_32_CKSUM};
use eolib::{
    data::{decode_number, encode_number, EoReader, EoSerialize, EoWriter},
    protocol::r#pub::{Ecf, EcfRecord},
};
use glob::glob;
use serde_json::Value;
use std::{fs::File, io::Read};

use bytes::Bytes;

use crate::SETTINGS;

use super::save_pub_file;

pub const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_CKSUM);

pub fn load_class_file() -> Result<Ecf, Box<dyn std::error::Error>> {
    if SETTINGS.server.generate_pub {
        load_json()
    } else {
        load_pub()
    }
}

fn load_json() -> Result<Ecf, Box<dyn std::error::Error>> {
    let mut ecf_file = Ecf::default();

    for entry in glob("pub/classes/*.json")? {
        let path = entry?;
        let mut file = File::open(path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;

        let v: Value = serde_json::from_str(&json)?;
        let record = EcfRecord {
            name: v["name"].as_str().unwrap_or_default().to_string(),
            parent_type: v["parent"].as_u64().unwrap_or(0) as i32,
            stat_group: v["statGroup"].as_u64().unwrap_or(0) as i32,
            str: v["str"].as_u64().unwrap_or(0) as i32,
            intl: v["intl"].as_u64().unwrap_or(0) as i32,
            wis: v["wis"].as_u64().unwrap_or(0) as i32,
            agi: v["agi"].as_u64().unwrap_or(0) as i32,
            con: v["con"].as_u64().unwrap_or(0) as i32,
            cha: v["cha"].as_u64().unwrap_or(0) as i32,
        };
        ecf_file.classes.push(record);
    }

    ecf_file.classes.push(EcfRecord {
        name: "eof".to_string(),
        ..Default::default()
    });

    let mut writer = EoWriter::new();
    ecf_file.serialize(&mut writer);
    let buf = writer.to_byte_array();

    let mut digest = CRC32.digest();
    digest.update(&buf[7..]);

    let checksum = digest.finalize();

    let encoded = encode_number(checksum as i32);

    ecf_file.rid = [
        decode_number(&encoded[0..=1]) as i32,
        decode_number(&encoded[2..=3]) as i32,
    ];

    save_pub_file(&ecf_file, "pub/dat001.ecf")?;

    Ok(ecf_file)
}

fn load_pub() -> Result<Ecf, Box<dyn std::error::Error>> {
    let mut file = File::open("pub/dat001.ecf")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let bytes = Bytes::from(buf);
    let reader = EoReader::new(bytes);
    Ok(Ecf::deserialize(&reader)?)
}
