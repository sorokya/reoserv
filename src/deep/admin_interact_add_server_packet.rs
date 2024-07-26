use eolib::data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter};

use super::DialogLine;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct AdminInteractAddServerPacket {
    pub lines: Vec<DialogLine>,
}

impl EoSerialize for AdminInteractAddServerPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let current_chunked_reading_mode = reader.get_chunked_reading_mode();

        let mut packet = Self::default();
        reader.set_chunked_reading_mode(true);

        while reader.remaining() > 0 {
            packet.lines.push(EoSerialize::deserialize(reader)?);
        }

        reader.set_chunked_reading_mode(current_chunked_reading_mode);

        Ok(packet)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        for line in &self.lines {
            line.serialize(writer)?;
        }

        Ok(())
    }
}
