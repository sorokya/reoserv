use eolib::data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter};

use super::AccountRecoverPinReply;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct LoginAcceptServerPacket {
    pub reply_code: AccountRecoverPinReply,
}

impl EoSerialize for LoginAcceptServerPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let mut packet = Self::default();
        packet.reply_code = AccountRecoverPinReply::from(reader.get_short());
        Ok(packet)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        writer.add_short(self.reply_code.into())?;
        Ok(())
    }
}
