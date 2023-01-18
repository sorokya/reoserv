use std::{
    io::{
        prelude::{Read, Seek},
        SeekFrom,
    },
    path::Path,
};

use eo::{
    data::{EOByte, Serializeable, StreamReader},
    pubs::InnFile,
};

pub async fn load_inn_file(
    path: String,
) -> Result<InnFile, Box<dyn std::error::Error + Send + Sync>> {
    let path = Path::new(&path);
    if Path::exists(path) {
        let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
        let mut inn_file = InnFile::default();

        let mut data_buf: Vec<EOByte> = Vec::new();
        raw_file.seek(SeekFrom::Start(0))?;
        raw_file.read_to_end(&mut data_buf)?;

        let reader = StreamReader::new(&data_buf);

        inn_file.deserialize(&reader);

        info!("{} inns loaded", inn_file.inns.len());
        Ok(inn_file)
    } else {
        warn!("Inn file not found: {}", path.display());
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Inn file not found",
        )))
    }
}
