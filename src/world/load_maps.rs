use std::{
    collections::HashMap,
    io::{
        prelude::{Read, Seek},
        SeekFrom,
    },
    path::Path,
};

use bytes::Bytes;
use eolib::{protocol::map::Emf, data::{EoReader, EoSerialize}};
use futures::{stream, StreamExt};
use mysql_async::Pool;

use crate::{map::MapHandle, SETTINGS};

use super::WorldHandle;

pub async fn load_maps(
    pool: Pool,
    world: WorldHandle,
) -> Result<HashMap<i32, MapHandle>, Box<dyn std::error::Error + Send + Sync>> {
    let max_id = SETTINGS.server.num_of_maps;
    let mut map_files: HashMap<i32, MapHandle> = HashMap::with_capacity(max_id as usize);
    let mut load_handles = vec![];
    for i in 1..=max_id {
        load_handles.push(load_map(i, pool.to_owned(), world.to_owned()));
    }

    let mut stream = stream::iter(load_handles).buffer_unordered(100);
    while let Some(load_result) = stream.next().await {
        match load_result {
            Ok(load_result) => {
                map_files.insert(load_result.0, load_result.1);
            }
            Err(err) => {
                warn!("Failed to load map: {}", err);
            }
        }
    }
    info!("{} maps loaded", map_files.len());

    Ok(map_files)
}

async fn load_map(
    id: i32,
    pool: Pool,
    world: WorldHandle,
) -> Result<(i32, MapHandle), Box<dyn std::error::Error + Send + Sync>> {
    let raw_path = format!("maps/{:0>5}.emf", id);
    let path = Path::new(&raw_path);
    
    if !Path::exists(path) {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Map file not found: {}", raw_path),
        )));
    }

    let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
    let file_size: u64 = raw_file.metadata()?.len();

    let mut data_buf: Vec<u8> = Vec::new();
    raw_file.seek(SeekFrom::Start(0))?;
    raw_file.read_to_end(&mut data_buf)?;

    let data_buf = Bytes::from(data_buf);

    let reader = EoReader::new(data_buf);

    let file = Emf::deserialize(&reader)?;

    Ok((
        id,
        MapHandle::new(id, file_size as i32, pool, file, world),
    ))
}
