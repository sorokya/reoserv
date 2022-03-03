use std::path::Path;

use eo::data::pubs::MasterFile;

pub async fn load_master_file(
    path: String,
) -> Result<MasterFile, Box<dyn std::error::Error + Send + Sync>> {
    let path = Path::new(&path);
    if Path::exists(path) {
        let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
        let mut master_file = MasterFile::new();
        master_file.read(&mut raw_file)?;
        info!("{} skill masters loaded", master_file.records.len());
        Ok(master_file)
    } else {
        warn!("Skill Master file not found: {}", path.display());
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Skill Master file not found",
        )))
    }
}
