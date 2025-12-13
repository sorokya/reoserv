use std::{
    collections::HashMap,
    io::{
        prelude::{Read, Seek},
        SeekFrom,
    },
    path::PathBuf,
};

use bytes::Bytes;
use eolib::{
    data::{EoReader, EoSerialize},
    protocol::map::Emf,
};
use futures::{stream, StreamExt};
use glob::glob;
use mysql_async::Pool;

use crate::map::MapHandle;

use super::WorldHandle;

pub async fn load_maps(
    pool: Pool,
    world: WorldHandle,
) -> Result<HashMap<i32, MapHandle>, Box<dyn std::error::Error + Send + Sync>> {
    let entries = glob("data/maps/*.emf")?;
    let mut map_files: HashMap<i32, MapHandle> = HashMap::new();
    let mut load_handles = vec![];
    for path in entries {
        let path = match path {
            Ok(ref path) => path,
            Err(e) => {
                error!("Failed to read path: {}", e);
                continue;
            }
        };

        load_handles.push(load_map(
            path.to_path_buf(),
            pool.to_owned(),
            world.to_owned(),
        ));
    }

    let mut stream = stream::iter(load_handles).buffer_unordered(100);
    while let Some(load_result) = stream.next().await {
        if let Some(load_result) = load_result {
            map_files.insert(load_result.0, load_result.1);
        }
    }

    info!("{} maps loaded", map_files.len());

    map_files.insert(
        0,
        MapHandle::new(0, 0, pool.to_owned(), Emf::default(), world.to_owned()),
    );

    Ok(map_files)
}

async fn load_map(path: PathBuf, pool: Pool, world: WorldHandle) -> Option<(i32, MapHandle)> {
    let file_name = match path.file_name() {
        Some(file_name) => match file_name.to_str() {
            Some(file_name) => file_name,
            None => return None,
        },
        None => return None,
    };

    let left_part = file_name.split('.').collect::<Vec<&str>>()[0];

    if left_part.chars().any(|c| !c.is_ascii_digit()) {
        return None;
    }

    let id = left_part.parse::<i32>().unwrap();

    let mut raw_file = match tokio::fs::File::open(path).await {
        Ok(file) => file.into_std().await,
        Err(e) => {
            error!("Failed to open file: {}", e);
            return None;
        }
    };

    let file_size: u64 = match raw_file.metadata() {
        Ok(metadata) => metadata.len(),
        Err(e) => {
            error!("Failed to get metadata: {}", e);
            return None;
        }
    };

    let mut data_buf: Vec<u8> = Vec::new();
    if let Err(e) = raw_file.seek(SeekFrom::Start(0)) {
        error!("Failed to seek file: {}", e);
        return None;
    }

    if let Err(e) = raw_file.read_to_end(&mut data_buf) {
        error!("Failed to read file: {}", e);
        return None;
    }

    let data_buf = Bytes::from(data_buf);

    let reader = EoReader::new(data_buf);

    let file = match Emf::deserialize(&reader) {
        Ok(file) => file,
        Err(e) => {
            error!("Failed to deserialize Emf: {}", e);
            return None;
        }
    };

    let map_handle = MapHandle::new(id, file_size as i32, pool, file, world);
    map_handle.load().await.expect("Failed to load map. Timeout");

    Some((id, map_handle))
}
