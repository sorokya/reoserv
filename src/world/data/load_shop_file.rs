use std::path::Path;

use eo::data::pubs::ShopFile;

pub async fn load_shop_file(
    path: String,
) -> Result<ShopFile, Box<dyn std::error::Error + Send + Sync>> {
    let path = Path::new(&path);
    if Path::exists(path) {
        let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
        let mut shop_file = ShopFile::new();
        shop_file.read(&mut raw_file)?;
        info!("{} shops loaded", shop_file.records.len());
        Ok(shop_file)
    } else {
        warn!("Shop file not found: {}", path.display());
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Shop file not found",
        )))
    }
}
