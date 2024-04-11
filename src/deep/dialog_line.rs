use eolib::data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct DialogLine {
    pub left: String,
    pub right: String,
}

impl DialogLine {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EoSerialize for DialogLine {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let current_chunked_reading_mode = reader.get_chunked_reading_mode();
        reader.set_chunked_reading_mode(true);

        let mut dialog = DialogLine::default();

        dialog.left = reader.get_string()?;
        reader.next_chunk()?;
        dialog.right = reader.get_string()?;
        reader.next_chunk()?;

        reader.set_chunked_reading_mode(current_chunked_reading_mode);

        Ok(dialog)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        writer.add_string(&self.left);
        writer.add_byte(0xff);
        writer.add_string(&self.right);
        writer.add_byte(0xff);
        Ok(())
    }
}
