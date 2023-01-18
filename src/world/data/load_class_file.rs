use std::{
    io::{
        prelude::{Read, Seek},
        SeekFrom,
    },
    path::Path,
};

use eo::{
    data::{EOByte, Serializeable, StreamReader},
    pubs::EcfFile,
};

pub async fn load_class_file(
    path: String,
) -> Result<EcfFile, Box<dyn std::error::Error + Send + Sync>> {
    let path = Path::new(&path);
    if Path::exists(path) {
        let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
        let mut class_file = EcfFile::default();

        let mut data_buf: Vec<EOByte> = Vec::new();
        raw_file.seek(SeekFrom::Start(0))?;
        raw_file.read_to_end(&mut data_buf)?;

        let reader = StreamReader::new(&data_buf);

        class_file.deserialize(&reader);

        info!("{} classes loaded", class_file.classes.len());
        Ok(class_file)
    } else {
        warn!("Class file not found: {}", path.display());
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Class file not found",
        )))
    }
}
