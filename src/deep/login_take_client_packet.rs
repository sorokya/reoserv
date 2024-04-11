use eolib::data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct LoginTakeClientPacket;

impl LoginTakeClientPacket {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EoSerialize for LoginTakeClientPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let packet = Self::new();
        reader.get_byte()?;
        Ok(packet)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        writer.add_byte(b'R');
        Ok(())
    }
}
