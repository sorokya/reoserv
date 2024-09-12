use std::{fs::File, io::Read};

use bytes::Bytes;
use eolib::{
    data::{EoReader, EoSerialize},
    protocol::r#pub::server::{
        ShopCraftIngredientRecord, ShopCraftRecord, ShopFile, ShopRecord, ShopTradeRecord,
    },
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

    for entry in glob("data/pub/shops/*.json")? {
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
                    max_amount: v["maxAmount"].as_u64().unwrap_or(0) as i32,
                })
                .collect(),
            crafts: crafts
                .iter()
                .map(|v| {
                    let mut ingredients: Vec<ShopCraftIngredientRecord> = Vec::with_capacity(4);

                    for i in 1..=4 {
                        ingredients.push(ShopCraftIngredientRecord {
                            item_id: v[&format!("ingredient{}ItemId", i)].as_u64().unwrap_or(0)
                                as i32,
                            amount: v[&format!("ingredient{}Amount", i)].as_u64().unwrap_or(0)
                                as i32,
                        })
                    }

                    ShopCraftRecord {
                        item_id: v["itemId"].as_u64().unwrap_or(0) as i32,
                        ingredients: [
                            ingredients[0].clone(),
                            ingredients[1].clone(),
                            ingredients[2].clone(),
                            ingredients[3].clone(),
                        ],
                    }
                })
                .collect(),
        });
    }

    save_pub_file(&shop_file, "data/pub/dts001.esf")?;

    Ok(shop_file)
}

fn load_pub() -> Result<ShopFile, Box<dyn std::error::Error>> {
    if let Ok(mut file) = File::open("data/pub/serv_shops.epf") {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let bytes = Bytes::from(buf);
        let reader = EoReader::new(bytes);

        if reader.get_fixed_string(3) != "ESF" {
            return Err("Invalid file".into());
        }

        reader.get_short();
        reader.get_short();

        let mut esf = ShopFile::default();

        let num_records = reader.get_short();

        esf.shops = Vec::with_capacity(num_records as usize);

        reader.get_char();

        for _ in 0..num_records {
            let mut record = ShopRecord {
                behavior_id: reader.get_short(),
                ..Default::default()
            };

            let name_length = reader.get_char();
            record.name = reader.get_fixed_string(name_length as usize);
            record.min_level = reader.get_short();
            record.max_level = reader.get_short();
            record.class_requirement = reader.get_char();

            let num_trades = reader.get_short();
            let num_crafts = reader.get_char();

            record.trades = Vec::with_capacity(num_trades as usize);
            record.crafts = Vec::with_capacity(num_crafts as usize);

            for _ in 0..num_trades {
                record.trades.push(ShopTradeRecord {
                    item_id: reader.get_short(),
                    buy_price: reader.get_three(),
                    sell_price: reader.get_three(),
                    max_amount: reader.get_char(),
                });
            }

            for _ in 0..num_crafts {
                record.crafts.push(ShopCraftRecord {
                    item_id: reader.get_short(),
                    ingredients: [
                        ShopCraftIngredientRecord {
                            item_id: reader.get_short(),
                            amount: reader.get_char(),
                        },
                        ShopCraftIngredientRecord {
                            item_id: reader.get_short(),
                            amount: reader.get_char(),
                        },
                        ShopCraftIngredientRecord {
                            item_id: reader.get_short(),
                            amount: reader.get_char(),
                        },
                        ShopCraftIngredientRecord {
                            item_id: reader.get_short(),
                            amount: reader.get_char(),
                        },
                    ],
                });
            }

            esf.shops.push(record);
        }

        return Ok(esf);
    }

    if let Ok(mut file) = File::open("data/pub/dts001.esf") {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let bytes = Bytes::from(buf);
        let reader = EoReader::new(bytes);
        return Ok(ShopFile::deserialize(&reader)?);
    }

    Ok(ShopFile::default())
}
