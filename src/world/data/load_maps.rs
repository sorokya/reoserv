use std::{collections::HashMap, path::Path};

use eo::data::{map::MapFile, EOShort};
use futures::{stream, StreamExt};

use crate::{map::MapHandle, SETTINGS, world::WorldHandle};

pub async fn load_maps(
    world_handle: WorldHandle,
) -> Result<HashMap<EOShort, MapHandle>, Box<dyn std::error::Error + Send + Sync>> {
    let max_id = SETTINGS.server.num_of_maps as EOShort;
    let mut map_files: HashMap<EOShort, MapHandle> = HashMap::with_capacity(max_id as usize);
    let mut load_handles = vec![];
    for i in 1..max_id + 1 {
        load_handles.push(load_map(i, world_handle.clone()));
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
    id: EOShort,
    world_handle: WorldHandle,
) -> Result<(EOShort, MapHandle), Box<dyn std::error::Error + Send + Sync>> {
    let raw_path = format!("maps/{:0>5}.emf", id);
    let path = Path::new(&raw_path);
    let mut file = MapFile::new();
    if Path::exists(path) {
        let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
        file.read(&mut raw_file)?;
    } else {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Map file not found: {}", raw_path),
        )));
    }

    Ok((id, MapHandle::new(id, file, world_handle)))
}
