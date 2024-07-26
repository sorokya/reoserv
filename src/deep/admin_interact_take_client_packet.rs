use eolib::data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter};

use super::LookupType;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct AdminInteractTakeClientPacket {
    pub lookup_type: LookupType,
    pub id: i32,
}

impl EoSerialize for AdminInteractTakeClientPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let mut packet = Self::default();
        packet.lookup_type = LookupType::from(reader.get_char());
        packet.id = reader.get_short();
        Ok(packet)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        writer.add_char(self.lookup_type.into())?;
        writer.add_short(self.id)?;
        Ok(())
    }
}
