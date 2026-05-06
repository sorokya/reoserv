use config::{Config, ConfigError, File};

use crate::SETTINGS;

#[derive(Debug, Deserialize)]
pub struct Lang {
    #[serde(default = "default_lang_announce_freeze")]
    pub announce_freeze: String,
    #[serde(default = "default_lang_announce_unfreeze")]
    pub announce_unfreeze: String,
    #[serde(default = "default_lang_announce_remove")]
    pub announce_remove: String,
    #[serde(default = "default_lang_announce_mute")]
    pub announce_mute: String,
    #[serde(default = "default_lang_announce_global")]
    pub announce_global: String,
    #[serde(default = "default_lang_global_locked")]
    pub global_locked: String,
    #[serde(default = "default_lang_wedding_start")]
    pub wedding_start: String,
    #[serde(default = "default_lang_wedding_one")]
    pub wedding_one: String,
    #[serde(default = "default_lang_wedding_two")]
    pub wedding_two: String,
    #[serde(default = "default_lang_wedding_do_you")]
    pub wedding_do_you: String,
    #[serde(default = "default_lang_wedding_i_do")]
    pub wedding_i_do: String,
    #[serde(default = "default_lang_wedding_three")]
    pub wedding_three: String,
    #[serde(default = "default_lang_wedding_four")]
    pub wedding_four: String,
    #[serde(default = "default_lang_wedding_five")]
    pub wedding_five: String,
    #[serde(default = "default_lang_wedding_end")]
    pub wedding_end: String,
    #[serde(default = "default_lang_wedding_error")]
    pub wedding_error: String,
    #[serde(default = "default_lang_evacuate_warning")]
    pub evacuate_warning: String,
    #[serde(default = "default_lang_evacuate_last_warning")]
    pub evacuate_last_warning: String,
}

fn default_lang_announce_freeze() -> String {
    "Attention!! {victim} movement has been frozen -{name}".to_string()
}
fn default_lang_announce_unfreeze() -> String {
    "Attention!! {victim} movement has been released -{name}".to_string()
}
fn default_lang_announce_remove() -> String {
    "Attention!! {victim} has been removed from the game -{name} [{method}]".to_string()
}
fn default_lang_announce_mute() -> String {
    "Attention!! {victim} has been muted -{name}".to_string()
}
fn default_lang_announce_global() -> String {
    "World communications changed to: {state} -{name}".to_string()
}
fn default_lang_global_locked() -> String {
    "This channel is temporarily disabled".to_string()
}
fn default_lang_wedding_start() -> String {
    "Very well, the ceremony will start in {delay} seconds.".to_string()
}
fn default_lang_wedding_one() -> String {
    "we are here at the invitation of {partner} and {name}, who have come before us to join together in marriage.".to_string()
}
fn default_lang_wedding_two() -> String {
    "their relationship is based on love, respect, and a determination to face the future together in health or sickness, in joy and sorrow.".to_string()
}
fn default_lang_wedding_do_you() -> String {
    "{partner}, do you take {name} to be your partner, and promise to love, comfort and stay together as long as you both shall live?".to_string()
}
fn default_lang_wedding_i_do() -> String {
    "Yes, i do".to_string()
}
fn default_lang_wedding_three() -> String {
    "Let these rings be given and received as a token of your affection, sincerity and trust in one another.".to_string()
}
fn default_lang_wedding_four() -> String {
    "Please place these rings on eachothers finger..".to_string()
}
fn default_lang_wedding_five() -> String {
    "{partner} and {name} have consented together in marriage. And are now partners for as long you both shall live.".to_string()
}
fn default_lang_wedding_end() -> String {
    "Congratulations to the couple!".to_string()
}
fn default_lang_wedding_error() -> String {
    "I'm sorry, something went wrong..".to_string()
}
fn default_lang_evacuate_warning() -> String {
    "Warning! - please leave this map in {seconds} seconds or be sent to jail.".to_string()
}
fn default_lang_evacuate_last_warning() -> String {
    "Last warning! - leave this map in {seconds} seconds or be sent to jail.".to_string()
}

impl Default for Lang {
    fn default() -> Self {
        Self {
            announce_freeze: default_lang_announce_freeze(),
            announce_unfreeze: default_lang_announce_unfreeze(),
            announce_remove: default_lang_announce_remove(),
            announce_mute: default_lang_announce_mute(),
            announce_global: default_lang_announce_global(),
            global_locked: default_lang_global_locked(),
            wedding_start: default_lang_wedding_start(),
            wedding_one: default_lang_wedding_one(),
            wedding_two: default_lang_wedding_two(),
            wedding_do_you: default_lang_wedding_do_you(),
            wedding_i_do: default_lang_wedding_i_do(),
            wedding_three: default_lang_wedding_three(),
            wedding_four: default_lang_wedding_four(),
            wedding_five: default_lang_wedding_five(),
            wedding_end: default_lang_wedding_end(),
            wedding_error: default_lang_wedding_error(),
            evacuate_warning: default_lang_evacuate_warning(),
            evacuate_last_warning: default_lang_evacuate_last_warning(),
        }
    }
}

impl Lang {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name(&format!(
                "config/lang/{}.ron",
                SETTINGS.load().server.lang
            )))
            .build()?;

        s.try_deserialize()
    }

    pub fn reload() -> Result<Self, ConfigError> {
        Self::new()
    }
}
