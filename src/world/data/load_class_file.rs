use std::path::Path;

use eo::data::pubs::ClassFile;

pub async fn load_class_file(
    path: String,
) -> Result<ClassFile, Box<dyn std::error::Error + Send + Sync>> {
    let path = Path::new(&path);
    if Path::exists(path) {
        let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
        let mut class_file = ClassFile::new();
        class_file.read(&mut raw_file)?;
        info!("{} classes loaded", class_file.records.len());
        Ok(class_file)
    } else {
        warn!("Class file not found: {}", path.display());
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Class file not found",
        )))
    }
}
