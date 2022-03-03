use std::path::Path;

use eo::data::pubs::ItemFile;

pub async fn load_item_file(
    path: String,
) -> Result<ItemFile, Box<dyn std::error::Error + Send + Sync>> {
    let path = Path::new(&path);
    if Path::exists(path) {
        let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
        let mut item_file = ItemFile::new();
        item_file.read(&mut raw_file)?;
        info!("{} items loaded", item_file.records.len());
        Ok(item_file)
    } else {
        warn!("Item file not found: {}", path.display());
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Item file not found",
        )))
    }
}
