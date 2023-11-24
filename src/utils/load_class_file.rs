use crc::{Crc, CRC_32_CKSUM};
use glob::glob;
use serde_json::Value;
use std::{fs::File, io::Read};

use bytes::Bytes;
use eo::{
    data::{
        decode_number, encode_number, EOChar, EOShort, Serializeable, StreamBuilder, StreamReader,
    },
    pubs::{EcfClass, EcfClassType, EcfFile},
};

use crate::SETTINGS;

pub const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_CKSUM);

pub fn load_class_file() -> Result<EcfFile, Box<dyn std::error::Error>> {
    if SETTINGS.server.generate_pub {
        load_json()
    } else {
        load_pub()
    }
}

fn load_json() -> Result<EcfFile, Box<dyn std::error::Error>> {
    let mut ecf_file = EcfFile::default();
    ecf_file.magic = "ECF".to_string();

    for entry in glob("pub/classes/*.json")? {
        let path = entry?;
        let mut file = File::open(path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;

        let v: Value = serde_json::from_str(&json)?;
        let record = EcfClass {
            name: v["name"].as_str().unwrap_or_default().to_string(),
            parent_type: v["parent"].as_u64().unwrap_or(0) as EOChar,
            r#type: EcfClassType::from_char(v["type"].as_u64().unwrap_or(0) as EOChar).unwrap(),
            str: v["str"].as_u64().unwrap_or(0) as EOShort,
            intl: v["intl"].as_u64().unwrap_or(0) as EOShort,
            wis: v["wis"].as_u64().unwrap_or(0) as EOShort,
            agi: v["agi"].as_u64().unwrap_or(0) as EOShort,
            con: v["con"].as_u64().unwrap_or(0) as EOShort,
            cha: v["cha"].as_u64().unwrap_or(0) as EOShort,
        };
        ecf_file.classes.push(record);
        ecf_file.num_classes += 1;
    }

    ecf_file.classes.push(EcfClass {
        name: "eof".to_string(),
        ..Default::default()
    });
    ecf_file.num_classes += 1;

    let mut builder = StreamBuilder::new();
    ecf_file.serialize(&mut builder);
    let buf = builder.get();

    let mut digest = CRC32.digest();
    digest.update(&buf[7..]);

    let checksum = digest.finalize();

    let encoded = encode_number(checksum);

    ecf_file.rid = [
        decode_number(&encoded[0..=1]) as EOShort,
        decode_number(&encoded[2..=3]) as EOShort,
    ];

    Ok(ecf_file)
}

fn load_pub() -> Result<EcfFile, Box<dyn std::error::Error>> {
    let mut file = File::open("pub/dat001.ecf")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let bytes = Bytes::from(buf);

    let reader = StreamReader::new(bytes);

    let mut ecf_file = EcfFile::default();
    ecf_file.deserialize(&reader);
    Ok(ecf_file)
}
