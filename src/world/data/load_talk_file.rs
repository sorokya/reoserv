use std::path::Path;

use eo::data::pubs::TalkFile;

pub async fn load_talk_file(
    path: String,
) -> Result<TalkFile, Box<dyn std::error::Error + Send + Sync>> {
    let path = Path::new(&path);
    if Path::exists(path) {
        let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
        let mut talk_file = TalkFile::new();
        talk_file.read(&mut raw_file)?;
        info!("{} speech records loaded", talk_file.records.len());
        Ok(talk_file)
    } else {
        warn!("Talk file not found: {}", path.display());
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Talk file not found",
        )))
    }
}
