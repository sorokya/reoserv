use std::path::Path;

use eo::data::pubs::SpellFile;

pub async fn load_spell_file(
    path: String,
) -> Result<SpellFile, Box<dyn std::error::Error + Send + Sync>> {
    let path = Path::new(&path);
    if Path::exists(path) {
        let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
        let mut spell_file = SpellFile::new();
        spell_file.read(&mut raw_file)?;
        info!("{} spells loaded", spell_file.records.len());
        Ok(spell_file)
    } else {
        warn!("Spell file not found: {}", path.display());
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Spell file not found",
        )))
    }
}
