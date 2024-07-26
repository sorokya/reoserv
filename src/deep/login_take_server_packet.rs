use eolib::data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter};

use super::AccountRecoverReply;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct LoginTakeServerPacket {
    pub reply_code: AccountRecoverReply,
}

impl EoSerialize for LoginTakeServerPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let mut packet = Self::default();
        packet.reply_code = AccountRecoverReply::from(reader.get_short());
        Ok(packet)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        writer.add_short(self.reply_code.into())?;
        Ok(())
    }
}
