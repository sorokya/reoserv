use eolib::data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct LoginTakeClientPacket;

impl EoSerialize for LoginTakeClientPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        reader.get_byte()?;
        Ok(Self::default())
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        writer.add_byte(b'R');
        Ok(())
    }
}
