use std::{fs::File, io::Read};

use bytes::Bytes;
use eolib::{
    data::{EoReader, EoSerialize},
    protocol::r#pub::server::{ShopCraftRecord, ShopFile, ShopRecord, ShopTradeRecord},
};
use glob::glob;
use serde_json::Value;

use crate::SETTINGS;

use super::save_pub_file;

pub fn load_shop_file() -> Result<ShopFile, Box<dyn std::error::Error>> {
    if SETTINGS.server.generate_pub {
        load_json()
    } else {
        load_pub()
    }
}

fn load_json() -> Result<ShopFile, Box<dyn std::error::Error>> {
    let mut shop_file = ShopFile::default();

    for entry in glob("pub/shops/*.json")? {
        let path = entry?;
        let mut file = File::open(path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;

        let v: Value = serde_json::from_str(&json)?;

        let trades = v["trades"].as_array().unwrap();
        let crafts = v["crafts"].as_array().unwrap();

        shop_file.shops.push(ShopRecord {
            behavior_id: v["behaviorId"].as_u64().unwrap_or(0) as i32,
            name: v["name"].as_str().unwrap_or_default().to_string(),
            min_level: v["minLevel"].as_u64().unwrap_or(0) as i32,
            max_level: v["maxLevel"].as_u64().unwrap_or(0) as i32,
            class_requirement: v["classRequirement"].as_u64().unwrap_or(0) as i32,
            trades: trades
                .iter()
                .map(|v| ShopTradeRecord {
                    item_id: v["itemId"].as_u64().unwrap_or(0) as i32,
                    buy_price: v["buyPrice"].as_u64().unwrap_or(0) as i32,
                    sell_price: v["sellPrice"].as_u64().unwrap_or(0) as i32,
                    max_amount: v["maxBuyAmount"].as_u64().unwrap_or(0) as i32,
                })
                .collect(),
            crafts: crafts
                .iter()
                .map(|v| ShopCraftRecord {
                    item_id: v["itemId"].as_u64().unwrap_or(0) as i32,
                    ingredient1_id: v["ingredient1Id"].as_u64().unwrap_or(0) as i32,
                    ingredient1_amount: v["ingredient1Amount"].as_u64().unwrap_or(0) as i32,
                    ingredient2_id: v["ingredient2Id"].as_u64().unwrap_or(0) as i32,
                    ingredient2_amount: v["ingredient2Amount"].as_u64().unwrap_or(0) as i32,
                    ingredient3_id: v["ingredient3Id"].as_u64().unwrap_or(0) as i32,
                    ingredient3_amount: v["ingredient3Amount"].as_u64().unwrap_or(0) as i32,
                    ingredient4_id: v["ingredient4Id"].as_u64().unwrap_or(0) as i32,
                    ingredient4_amount: v["ingredient4Amount"].as_u64().unwrap_or(0) as i32,
                })
                .collect(),
        });
    }

    save_pub_file(&shop_file, "pub/dts001.esf")?;

    Ok(shop_file)
}

fn load_pub() -> Result<ShopFile, Box<dyn std::error::Error>> {
    let mut file = File::open("pub/dts001.esf")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let bytes = Bytes::from(buf);
    let reader = EoReader::new(bytes);
    Ok(ShopFile::deserialize(&reader)?)
}
