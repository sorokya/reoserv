use std::{fs::File, io::Read};

use bytes::Bytes;
use eo::{
    data::{EOChar, EOInt, EOShort, Serializeable, StreamReader},
    pubs::{Craft, Shop, ShopFile, Trade},
};
use glob::glob;
use serde_json::Value;

use crate::SETTINGS;

pub fn load_shop_file() -> Result<ShopFile, Box<dyn std::error::Error>> {
    if SETTINGS.server.generate_pub {
        load_json()
    } else {
        load_pub()
    }
}

fn load_json() -> Result<ShopFile, Box<dyn std::error::Error>> {
    let mut shop_file = ShopFile::default();
    shop_file.magic = "ESF".to_string();

    for entry in glob("pub/shops/*.json")? {
        let path = entry?;
        let mut file = File::open(path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;

        let v: Value = serde_json::from_str(&json)?;

        let trades = v["trades"].as_array().unwrap();
        let crafts = v["crafts"].as_array().unwrap();

        shop_file.shops.push(Shop {
            vendor_id: v["behaviorId"].as_u64().unwrap_or(0) as EOShort,
            name: v["name"].as_str().unwrap_or_default().to_string(),
            min_level: v["minLevel"].as_u64().unwrap_or(0) as EOChar,
            max_level: v["maxLevel"].as_u64().unwrap_or(0) as EOChar,
            class_req: v["classReq"].as_u64().unwrap_or(0) as EOChar,
            num_trades: trades.len() as EOShort,
            num_crafts: crafts.len() as EOChar,
            trades: trades
                .iter()
                .map(|v| Trade {
                    item_id: v["itemId"].as_u64().unwrap_or(0) as EOShort,
                    buy_price: v["buyPrice"].as_u64().unwrap_or(0) as EOInt,
                    sell_price: v["sellPrice"].as_u64().unwrap_or(0) as EOInt,
                    max_amount: v["maxBuyAmount"].as_u64().unwrap_or(0) as EOChar,
                })
                .collect(),
            crafts: crafts
                .iter()
                .map(|v| Craft {
                    item_id: v["itemId"].as_u64().unwrap_or(0) as EOShort,
                    ingredient1_item_id: v["ingredient1ItemId"].as_u64().unwrap_or(0) as EOShort,
                    ingredient1_amount: v["ingredient1Amount"].as_u64().unwrap_or(0) as EOChar,
                    ingredient2_item_id: v["ingredient2ItemId"].as_u64().unwrap_or(0) as EOShort,
                    ingredient2_amount: v["ingredient2Amount"].as_u64().unwrap_or(0) as EOChar,
                    ingredient3_item_id: v["ingredient3ItemId"].as_u64().unwrap_or(0) as EOShort,
                    ingredient3_amount: v["ingredient3Amount"].as_u64().unwrap_or(0) as EOChar,
                    ingredient4_item_id: v["ingredient4ItemId"].as_u64().unwrap_or(0) as EOShort,
                    ingredient4_amount: v["ingredient4Amount"].as_u64().unwrap_or(0) as EOChar,
                })
                .collect(),
        });
    }

    Ok(shop_file)
}

fn load_pub() -> Result<ShopFile, Box<dyn std::error::Error>> {
    let mut file = File::open("pub/dts001.esf")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let bytes = Bytes::from(buf);
    let reader = StreamReader::new(bytes);

    let mut shop_file = ShopFile::default();
    shop_file.deserialize(&reader);
    Ok(shop_file)
}
