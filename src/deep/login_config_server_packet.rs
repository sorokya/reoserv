use eolib::data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct LoginConfigServerPacket {
    pub max_skins: i32,
    pub max_hair_modals: i32,
    pub max_character_name: i32,
}

impl EoSerialize for LoginConfigServerPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let mut packet = Self::default();
        packet.max_skins = reader.get_short()?;
        packet.max_hair_modals = reader.get_short()?;
        packet.max_character_name = reader.get_char()?;
        Ok(packet)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        writer.add_short(self.max_skins)?;
        writer.add_short(self.max_hair_modals)?;
        writer.add_char(self.max_character_name)?;
        Ok(())
    }
}
