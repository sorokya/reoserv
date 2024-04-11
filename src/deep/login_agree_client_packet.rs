use eolib::data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct LoginAgreeClientPacket {
    pub account_name: String,
    pub pin: String,
    pub password: String,
}

impl LoginAgreeClientPacket {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EoSerialize for LoginAgreeClientPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let current_chunked_reading_mode = reader.get_chunked_reading_mode();
        reader.set_chunked_reading_mode(true);
        let mut packet = Self::new();
        packet.account_name = reader.get_string()?;
        reader.next_chunk()?;
        packet.pin = reader.get_string()?;
        reader.next_chunk()?;
        packet.password = reader.get_string()?;
        Ok(packet)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        writer.add_string(&self.account_name);
        writer.add_byte(0xff);
        writer.add_string(&self.pin);
        writer.add_byte(0xff);
        writer.add_string(&self.password);
        Ok(())
    }
}
