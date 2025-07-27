use std::cmp;

use config::{Config, ConfigError, File};
use eolib::{data::THREE_MAX, protocol::r#pub::server::DropRecord};

#[derive(Debug, Deserialize)]
pub struct GlobalDrops {
    pub drops: Vec<DropRecord>,
}

impl GlobalDrops {
    pub fn new() -> Result<Self, ConfigError> {
        let config_result = Config::builder()
            .add_source(File::with_name("config/GlobalDrops.ron").required(false))
            .add_source(File::with_name("config/GlobalDrops.local.ron").required(false))
            .build();

        let mut s: GlobalDrops = match config_result {
            Ok(config) => match config.try_deserialize() {
                Ok(global_drops) => global_drops,
                Err(_) => GlobalDrops { drops: Vec::new() },
            },
            Err(_) => GlobalDrops { drops: Vec::new() },
        };

        for drop in s.drops.iter_mut() {
            drop.rate = drop.rate.clamp(0, 100);
            drop.min_amount = drop.min_amount.clamp(0, THREE_MAX);
            drop.max_amount = drop.max_amount.clamp(0, THREE_MAX);
            drop.min_amount = cmp::min(drop.min_amount, drop.max_amount);
            drop.max_amount = cmp::max(drop.min_amount, drop.max_amount);
            drop.rate *= 640;
        }

        Ok(s)
    }
}
