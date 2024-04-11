use eolib::data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter};

use super::AccountRecoverReply;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct LoginUseServerPacket {
    pub reply_code: AccountRecoverReply,
    pub email_address: Option<String>,
}

impl LoginUseServerPacket {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EoSerialize for LoginUseServerPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let current_chunked_reading_mode = reader.get_chunked_reading_mode();
        reader.set_chunked_reading_mode(true);
        let mut packet = Self::new();
        packet.reply_code = AccountRecoverReply::from(reader.get_short()?);
        reader.next_chunk();
        if reader.remaining()? > 0 {
            packet.email_address = Some(reader.get_string()?);
        }
        reader.set_chunked_reading_mode(current_chunked_reading_mode);
        Ok(packet)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        writer.add_short(self.reply_code.into())?;
        if let Some(email) = &self.email_address {
            writer.add_byte(0xff);
            writer.add_string(email);
        }
        Ok(())
    }
}
