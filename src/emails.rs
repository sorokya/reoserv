use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize)]
pub struct EmailTemplate {
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct Emails {
    pub validation: EmailTemplate,
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
}
