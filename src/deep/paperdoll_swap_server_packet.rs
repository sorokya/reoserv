use eolib::{
    data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter},
    protocol::net::server::{AvatarChange, CharacterStatsEquipmentChange, EquipmentChange},
};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct PaperdollSwapServerPacket {
    pub change: AvatarChange,
    pub item_id: i32,
    pub remaining_amount: i32,
    pub removed_item_id: i32,
    pub removed_item_amount: i32,
    pub stats: CharacterStatsEquipmentChange,
}

impl EoSerialize for PaperdollSwapServerPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let current_chunked_reading_mode = reader.get_chunked_reading_mode();
        let mut data = Self::default();
        data.change = EoSerialize::deserialize(reader)?;
        data.item_id = reader.get_short()?;
        data.remaining_amount = reader.get_three()?;
        reader.get_char()?;
        data.removed_item_id = reader.get_short()?;
        data.removed_item_amount = reader.get_three()?;
        data.stats = EoSerialize::deserialize(reader)?;
        reader.set_chunked_reading_mode(current_chunked_reading_mode);
        Ok(data)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        EoSerialize::serialize(&self.change, writer)?;
        writer.add_short(self.item_id)?;
        writer.add_three(self.remaining_amount)?;
        writer.add_char(0)?;
        writer.add_short(self.removed_item_id)?;
        writer.add_three(self.removed_item_amount)?;
        EoSerialize::serialize(&self.stats, writer)?;
        Ok(())
    }
}
