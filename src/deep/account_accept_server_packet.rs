use eolib::data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter};

use super::AccountValidationReply;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct AccountAcceptServerPacket {
    pub reply_code: AccountValidationReply,
}

impl EoSerialize for AccountAcceptServerPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let mut packet = Self::default();
        packet.reply_code = AccountValidationReply::from(reader.get_short());
        Ok(packet)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        writer.add_short(self.reply_code.into())?;
        Ok(())
    }
}
