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
    #[serde(default = "default_formulas_hp")]
    pub hp: String,
    #[serde(default = "default_formulas_tp")]
    pub tp: String,
    #[serde(default = "default_formulas_sp")]
    pub sp: String,
    #[serde(default = "default_formulas_max_weight")]
    pub max_weight: String,
    #[serde(default = "default_formulas_hit_rate")]
    pub hit_rate: String,
    #[serde(default = "default_formulas_damage")]
    pub damage: String,
    #[serde(default = "default_formulas_party_exp_share")]
    pub party_exp_share: String,
    #[serde(default)]
    pub classes: Vec<ClassFormulas>,
}

fn default_formulas_hp() -> String {
    "10.0 + (2.5 * level) + (2.5 * con)".to_string()
}
fn default_formulas_tp() -> String {
    "10.0 + (2.5 * level) + (2.5 * int) + (1.5 * wis)".to_string()
}
fn default_formulas_sp() -> String {
    "20.0 + (2.0 * level)".to_string()
}
fn default_formulas_max_weight() -> String {
    "70.0 + str".to_string()
}
fn default_formulas_hit_rate() -> String {
    "if(target_sitting, 1.0, min(0.8, max(0.5, if(accuracy + target_evade == 0.0, 0.5, accuracy / (target_evade * 2.0)))))".to_string()
}
fn default_formulas_damage() -> String {
    "if(critical, 1.5, 1.0) * max(1, if(damage >= target_armor * 2.0, damage, damage * math::pow((damage / (target_armor * 2.0), 2.0))))".to_string()
}
fn default_formulas_party_exp_share() -> String {
    "if(members > 2, floor(exp * ((1 + members) / members)), floor(exp / 2))".to_string()
}

impl Default for Formulas {
    fn default() -> Self {
        Self {
            hp: default_formulas_hp(),
            tp: default_formulas_tp(),
            sp: default_formulas_sp(),
            max_weight: default_formulas_max_weight(),
            hit_rate: default_formulas_hit_rate(),
            damage: default_formulas_damage(),
            party_exp_share: default_formulas_party_exp_share(),
            classes: vec![],
        }
    }
}

impl Formulas {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config/Formulas.ron"))
            .add_source(File::with_name("config/Formulas.local.ron").required(false))
            .build()?;

        s.try_deserialize()
    }

    pub fn reload() -> Result<Self, ConfigError> {
        Self::new()
    }
}
