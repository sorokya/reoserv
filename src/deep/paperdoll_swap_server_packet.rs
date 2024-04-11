use eolib::{
    data::{EoReader, EoReaderError, EoSerialize, EoSerializeError, EoWriter},
    protocol::net::server::{CharacterStatsEquipmentChange, EquipmentChange},
};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct PaperdollSwapServerPacket {
    pub player_id: i32,
    pub equipment: EquipmentChange,
    pub equiped_item_id: i32,
    pub equiped_item_amount: i32,
    pub removed_item_id: i32,
    pub removed_item_amount: i32,
    pub stats: CharacterStatsEquipmentChange,
}

impl PaperdollSwapServerPacket {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EoSerialize for PaperdollSwapServerPacket {
    fn deserialize(reader: &EoReader) -> Result<Self, EoReaderError> {
        let mut packet = Self::new();
        packet.player_id = reader.get_short()?;
        reader.get_char();
        reader.get_char();
        packet.equipment = EoSerialize::deserialize(reader)?;
        packet.equiped_item_id = reader.get_short()?;
        packet.equiped_item_amount = reader.get_three()?;
        packet.removed_item_id = reader.get_short()?;
        packet.removed_item_amount = reader.get_three()?;
        packet.stats = EoSerialize::deserialize(reader)?;
        Ok(packet)
    }

    fn serialize(&self, writer: &mut EoWriter) -> Result<(), EoSerializeError> {
        writer.add_short(self.player_id)?;
        writer.add_char(1)?;
        writer.add_char(0)?;
        self.equipment.serialize(writer)?;
        writer.add_short(self.equiped_item_id)?;
        writer.add_three(self.equiped_item_amount)?;
        writer.add_short(self.removed_item_id)?;
        writer.add_three(self.removed_item_amount)?;
        self.stats.serialize(writer)?;
        Ok(())
    }
}
