use std::{
    io::{
        prelude::{Read, Seek},
        SeekFrom,
    },
    path::Path,
};

use eo::{
    data::{EOByte, Serializeable, StreamReader},
    pubs::ShopFile,
};

pub async fn load_shop_file(
    path: String,
) -> Result<ShopFile, Box<dyn std::error::Error + Send + Sync>> {
    let path = Path::new(&path);
    if Path::exists(path) {
        let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
        let mut shop_file = ShopFile::default();

        let mut data_buf: Vec<EOByte> = Vec::new();
        raw_file.seek(SeekFrom::Start(0))?;
        raw_file.read_to_end(&mut data_buf)?;

        let reader = StreamReader::new(&data_buf);

        shop_file.deserialize(&reader);

        info!("{} shops loaded", shop_file.shops.len());
        Ok(shop_file)
    } else {
        warn!("Shop file not found: {}", path.display());
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Shop file not found",
        )))
    }
}
