use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize)]
pub struct ClassFormulas {
    pub damage: String,
    pub accuracy: String,
    pub evade: String,
    pub defense: String,
}

#[derive(Debug, Deserialize)]
pub struct Formulas {
    pub hp: String,
    pub tp: String,
    pub sp: String,
    pub max_weight: String,
    pub hit_rate: String,
    pub damage: String,
    pub party_exp_share: String,
    pub classes: Vec<ClassFormulas>,
}

impl Formulas {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config/Formulas.ron"))
            .add_source(File::with_name("config/Formulas.local.ron").required(false))
            .build()?;

        s.try_deserialize()
    }
}
