use eolib::data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct AccountAcceptClientPacket {
    pub sequence_number: i32,
    pub account_name: String,
    pub email_address: String,
}

impl EoSerialize for AccountAcceptClientPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let current_chunked_reading_mode = reader.get_chunked_reading_mode();
        reader.set_chunked_reading_mode(true);
        let mut packet = Self::default();
        packet.sequence_number = reader.get_short()?;
        reader.next_chunk()?;
        packet.account_name = reader.get_string()?;
        reader.next_chunk()?;
        packet.email_address = reader.get_string()?;
        reader.set_chunked_reading_mode(current_chunked_reading_mode);
        Ok(packet)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        writer.add_short(self.sequence_number)?;
        writer.add_byte(0xff);
        writer.add_string(&self.account_name);
        writer.add_byte(0xff);
        writer.add_string(&self.email_address);
        Ok(())
    }
}
