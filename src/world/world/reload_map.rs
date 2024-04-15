use std::{
    io::{Read, Seek, SeekFrom},
    path::Path,
};

use bytes::Bytes;
use eolib::{
    data::{EoReader, EoSerialize},
    protocol::map::Emf,
};
use tokio::fs::File;

use super::World;

impl World {
    pub async fn reload_map(&mut self, map_id: i32) {
        let maps = match self.maps.as_mut() {
            Some(maps) => maps,
            None => return,
        };

        let map = match maps.get_mut(&map_id) {
            Some(map) => map,
            None => return,
        };

        let raw_path = format!("data/maps/{:0>5}.emf", map_id);
        let path = Path::new(&raw_path);

        if !Path::exists(path) {
            error!("Map file not found: {}", raw_path);
            return;
        }

        let mut raw_file = match File::open(path).await {
            Ok(file) => file.into_std().await,
            Err(e) => {
                error!("Failed to open file: {}", e);
                return;
            }
        };

        let file_size = match raw_file.metadata() {
            Ok(meta) => meta.len(),
            Err(e) => {
                error!("Failed to get file metadata: {}", e);
                return;
            }
        };

        let mut data_buf: Vec<u8> = Vec::new();
        if let Err(e) = raw_file.seek(SeekFrom::Start(0)) {
            error!("Failed to seek to file start: {}", e);
            return;
        }

        if let Err(e) = raw_file.read_to_end(&mut data_buf) {
            error!("Failed to read file: {}", e);
            return;
        }

        let data_buf = Bytes::from(data_buf);

        let reader = EoReader::new(data_buf);

        let file = match Emf::deserialize(&reader) {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to deserialize Emf: {}", e);
                return;
            }
        };

        map.reload(Box::new(file), file_size as i32);
    }
}
