use std::path::Path;

use eo::data::pubs::DropFile;

pub async fn load_drop_file(
    path: String,
) -> Result<DropFile, Box<dyn std::error::Error + Send + Sync>> {
    let path = Path::new(&path);
    if Path::exists(path) {
        let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
        let mut drop_file = DropFile::new();
        drop_file.read(&mut raw_file)?;
        info!("{} drops loaded", drop_file.records.len());
        Ok(drop_file)
    } else {
        warn!("Drop file not found: {}", path.display());
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Drop file not found",
        )))
    }
}
