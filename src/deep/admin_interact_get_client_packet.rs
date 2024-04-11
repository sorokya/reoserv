use eolib::data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter};

use super::LookupType;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct AdminInteractGetClientPacket {
    pub lookup_type: LookupType,
    pub id: i32,
}

impl AdminInteractGetClientPacket {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EoSerialize for AdminInteractGetClientPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let mut packet = Self::new();
        packet.lookup_type = LookupType::from(reader.get_char()?);
        packet.id = reader.get_short()?;
        Ok(packet)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        Ok(())
    }
}
