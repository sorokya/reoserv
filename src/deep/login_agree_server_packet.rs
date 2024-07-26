use eolib::data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter};

use super::AccountRecoverUpdateReply;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct LoginAgreeServerPacket {
    pub reply_code: AccountRecoverUpdateReply,
}

impl EoSerialize for LoginAgreeServerPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let mut packet = Self::default();
        packet.reply_code = AccountRecoverUpdateReply::from(reader.get_short());
        Ok(packet)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        writer.add_short(self.reply_code.into())?;
        Ok(())
    }
}
