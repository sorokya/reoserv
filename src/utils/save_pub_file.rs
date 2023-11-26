use std::io::Write;

use eo::data::{Serializeable, StreamBuilder};

pub fn save_pub_file<T: Serializeable>(
    file: &T,
    path: &str,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut builder = StreamBuilder::new();
    file.serialize(&mut builder);
    let buf = builder.get();

    let mut file = std::fs::File::create(path)?;
    file.write_all(&buf)?;

    Ok(())
}
