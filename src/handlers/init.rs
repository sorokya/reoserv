use crate::{PacketBuf, Tx};
use eo::{data::{Serializeable, StreamReader}, net::packets::client::init::Init};

pub fn init(buf: PacketBuf, tx: &Tx) -> Result<(), Box<dyn std::error::Error>> {
    let mut packet = Init::default();
    let reader = StreamReader::new(&buf);
    reader.seek(2);
    packet.deserialize(&reader);
    debug!("{:?}", packet);
    Ok(())
}