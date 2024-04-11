use eolib::data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct CaptchaCloseServerPacket {
    pub experience: i32,
}

impl CaptchaCloseServerPacket {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EoSerialize for CaptchaCloseServerPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let mut packet = Self::new();
        packet.experience = reader.get_int()?;
        Ok(packet)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        writer.add_int(self.experience)?;
        Ok(())
    }
}
