use std::path::Path;

use eo::data::pubs::InnFile;

pub async fn load_inn_file(
    path: String,
) -> Result<InnFile, Box<dyn std::error::Error + Send + Sync>> {
    let path = Path::new(&path);
    if Path::exists(path) {
        let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
        let mut inn_file = InnFile::new();
        inn_file.read(&mut raw_file)?;
        info!("{} inns loaded", inn_file.records.len());
        Ok(inn_file)
    } else {
        warn!("Inn file not found: {}", path.display());
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Inn file not found",
        )))
    }
}
