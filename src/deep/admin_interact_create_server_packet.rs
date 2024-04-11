use eolib::data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter};

use super::DialogLine;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct AdminInteractCreateServerPacket {
    pub title: String,
    pub lines: Vec<DialogLine>,
}

impl AdminInteractCreateServerPacket {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EoSerialize for AdminInteractCreateServerPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let current_chunked_reading_mode = reader.get_chunked_reading_mode();

        let mut packet = Self::new();
        reader.set_chunked_reading_mode(true);

        packet.title = reader.get_string()?;
        reader.next_chunk()?;

        while reader.remaining()? > 0 {
            packet.lines.push(EoSerialize::deserialize(reader)?);
        }

        reader.set_chunked_reading_mode(current_chunked_reading_mode);

        Ok(packet)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        writer.add_string(&self.title);
        writer.add_byte(0xff);

        for line in self.lines {
            line.serialize(writer)?;
        }

        Ok(())
    }
}
