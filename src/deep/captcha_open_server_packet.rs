use eolib::data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct CaptchaOpenServerPacket {
    pub id: i32,
    pub reward_exp: i32,
    pub captcha: Option<String>,
}

impl EoSerialize for CaptchaOpenServerPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let current_chunked_reading_mode = reader.get_chunked_reading_mode();
        reader.set_chunked_reading_mode(true);
        let mut packet = Self::default();
        packet.id = reader.get_short();
        packet.reward_exp = reader.get_three();
        if reader.next_chunk().is_ok() {
            packet.captcha = Some(reader.get_string());
        }
        reader.set_chunked_reading_mode(current_chunked_reading_mode);
        Ok(packet)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        writer.add_short(self.id)?;
        writer.add_three(self.reward_exp)?;
        if let Some(captcha) = &self.captcha {
            writer.add_byte(0xff);
            writer.add_string(captcha);
        }
        Ok(())
    }
}
