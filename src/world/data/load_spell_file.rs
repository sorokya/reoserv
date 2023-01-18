use std::{
    io::{
        prelude::{Read, Seek},
        SeekFrom,
    },
    path::Path,
};

use eo::{
    data::{EOByte, Serializeable, StreamReader},
    pubs::EsfFile,
};

pub async fn load_spell_file(
    path: String,
) -> Result<EsfFile, Box<dyn std::error::Error + Send + Sync>> {
    let path = Path::new(&path);
    if Path::exists(path) {
        let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
        let mut spell_file = EsfFile::default();

        let mut data_buf: Vec<EOByte> = Vec::new();
        raw_file.seek(SeekFrom::Start(0))?;
        raw_file.read_to_end(&mut data_buf)?;

        let reader = StreamReader::new(&data_buf);

        spell_file.deserialize(&reader);

        info!("{} spells loaded", spell_file.spells.len());
        Ok(spell_file)
    } else {
        warn!("Spell file not found: {}", path.display());
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Spell file not found",
        )))
    }
}
