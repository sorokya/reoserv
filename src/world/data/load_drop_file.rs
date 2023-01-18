use std::{
    io::{
        prelude::{Read, Seek},
        SeekFrom,
    },
    path::Path,
};

use eo::{
    data::{EOByte, Serializeable, StreamReader},
    pubs::DropFile,
};

pub async fn load_drop_file(
    path: String,
) -> Result<DropFile, Box<dyn std::error::Error + Send + Sync>> {
    let path = Path::new(&path);
    if Path::exists(path) {
        let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
        let mut drop_file = DropFile::default();

        let mut data_buf: Vec<EOByte> = Vec::new();
        raw_file.seek(SeekFrom::Start(0))?;
        raw_file.read_to_end(&mut data_buf)?;

        let reader = StreamReader::new(&data_buf);

        drop_file.deserialize(&reader);

        info!("{} drops loaded", drop_file.npcs.len());
        Ok(drop_file)
    } else {
        warn!("Drop file not found: {}", path.display());
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Drop file not found",
        )))
    }
}
