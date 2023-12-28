use std::io::Write;

use eolib::data::{EoSerialize, EoWriter};

pub fn save_pub_file<T: EoSerialize>(
    file: &T,
    path: &str,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut writer = EoWriter::new();
    file.serialize(&mut writer);
    let buf = writer.to_byte_array();

    let mut file = std::fs::File::create(path)?;
    file.write_all(&buf)?;

    Ok(())
}
