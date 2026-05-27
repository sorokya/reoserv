use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize, Default)]
pub struct EmailTemplate {
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub body: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct Emails {
    #[serde(default)]
    pub validation: EmailTemplate,
    #[serde(default)]
    pub recovery: EmailTemplate,
}

impl Emails {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config/Emails.ron"))
            .add_source(File::with_name("config/Emails.local.ron").required(false))
            .build()?;

        s.try_deserialize()
    }

    pub fn reload() -> Result<Self, ConfigError> {
        Self::new()
    }
}
