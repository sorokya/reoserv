use eolib::data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct LoginUseClientPacket {
    pub account_name: String,
}

impl LoginUseClientPacket {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EoSerialize for LoginUseClientPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let mut packet = Self::new();
        packet.account_name = reader.get_string()?;
        Ok(packet)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        writer.add_string(&self.account_name);
        Ok(())
    }
}
