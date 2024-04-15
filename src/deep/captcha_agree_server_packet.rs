use eolib::data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct CaptchaAgreeServerPacket {
    pub id: i32,
    pub captcha: String,
}

impl EoSerialize for CaptchaAgreeServerPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let mut packet = Self::default();
        packet.id = reader.get_short()?;
        packet.captcha = reader.get_string()?;
        Ok(packet)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        writer.add_short(self.id)?;
        writer.add_string(&self.captcha);
        Ok(())
    }
}
