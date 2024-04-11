use eolib::data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct LoginAcceptClientPacket {
    pub pin: String,
}

impl LoginAcceptClientPacket {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EoSerialize for LoginAcceptClientPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let mut packet = Self::new();
        packet.pin = reader.get_string()?;
        Ok(packet)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        writer.add_string(&self.pin);
        Ok(())
    }
}
