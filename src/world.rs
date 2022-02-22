use eo::data::{
    map::MapFile,
    pubs::{
        ClassFile, DropFile, InnFile, ItemFile, MasterFile, NPCFile, ShopFile, SpellFile, TalkFile,
    },
    EOShort,
};
use futures::stream::{self, StreamExt};
use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};

#[derive(Debug)]
pub struct World {
    pub maps: Arc<Mutex<HashMap<EOShort, MapFile>>>,
    pub class_file: Arc<Mutex<ClassFile>>,
    pub drop_file: Arc<Mutex<DropFile>>,
    pub inn_file: Arc<Mutex<InnFile>>,
    pub item_file: Arc<Mutex<ItemFile>>,
    pub master_file: Arc<Mutex<MasterFile>>,
    pub npc_file: Arc<Mutex<NPCFile>>,
    pub shop_file: Arc<Mutex<ShopFile>>,
    pub spell_file: Arc<Mutex<SpellFile>>,
    pub talk_file: Arc<Mutex<TalkFile>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            maps: Arc::new(Mutex::new(HashMap::new())),
            class_file: Arc::new(Mutex::new(ClassFile::default())),
            drop_file: Arc::new(Mutex::new(DropFile::default())),
            inn_file: Arc::new(Mutex::new(InnFile::default())),
            item_file: Arc::new(Mutex::new(ItemFile::default())),
            master_file: Arc::new(Mutex::new(MasterFile::default())),
            npc_file: Arc::new(Mutex::new(NPCFile::default())),
            shop_file: Arc::new(Mutex::new(ShopFile::default())),
            spell_file: Arc::new(Mutex::new(SpellFile::default())),
            talk_file: Arc::new(Mutex::new(TalkFile::default())),
        }
    }

    pub async fn load_maps(&mut self, max_id: EOShort) -> Result<(), Box<dyn std::error::Error>> {
        let mut load_handles = vec![];
        for i in 1..max_id + 1 {
            load_handles.push(self.load_map(i));
        }

        let mut stream = stream::iter(load_handles).buffer_unordered(100);
        let maps = self.maps.clone();
        while let Some(load_result) = stream.next().await {
            match load_result {
                Ok(load_result) => {
                    maps.lock()
                        .expect("Failed to get lock on maps")
                        .insert(load_result.0, load_result.1);
                },
                Err(err) => {
                    warn!("Failed to load map: {}", err);
                }
            }
        }
        info!(
            "{} maps loaded",
            maps.lock().expect("Failed to get lock on maps").len()
        );

        Ok(())
    }

    pub async fn load_pub_files(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let _ = tokio::join!(
            self.load_class_file("pub/dat001.ecf".to_string()),
            self.load_drop_file("pub/dtd001.edf".to_string()),
            self.load_inn_file("pub/din001.eid".to_string()),
            self.load_item_file("pub/dat001.eif".to_string()),
            self.load_master_file("pub/dsm001.emf".to_string()),
            self.load_npc_file("pub/dtn001.enf".to_string()),
            self.load_shop_file("pub/dts001.esf".to_string()),
            self.load_spell_file("pub/dsl001.esf".to_string()),
            self.load_talk_file("pub/ttd001.etf".to_string()),
        );
        Ok(())
    }

    async fn load_map(
        &self,
        id: EOShort,
    ) -> Result<(EOShort, MapFile), Box<dyn std::error::Error>> {
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

        Ok((id, file))
    }

    async fn load_class_file(&self, path: String) -> Result<(), Box<dyn std::error::Error + '_>> {
        let path = Path::new(&path);
        if Path::exists(path) {
            let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
            let mut class_file = self.class_file.lock()?;
            class_file.read(&mut raw_file)?;
            info!("{} classes loaded", class_file.records.len());
        } else {
            warn!("Class file not found: {}", path.display());
        }
        Ok(())
    }

    async fn load_drop_file(&self, path: String) -> Result<(), Box<dyn std::error::Error + '_>> {
        let path = Path::new(&path);
        if Path::exists(path) {
            let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
            let mut drop_file = self.drop_file.lock()?;
            drop_file.read(&mut raw_file)?;
            info!("{} drops loaded", drop_file.records.len());
        } else {
            warn!("Drop file not found: {}", path.display());
        }
        Ok(())
    }

    async fn load_inn_file(&self, path: String) -> Result<(), Box<dyn std::error::Error + '_>> {
        let path = Path::new(&path);
        if Path::exists(path) {
            let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
            let mut inn_file = self.inn_file.lock()?;
            inn_file.read(&mut raw_file)?;
            info!("{} inns loaded", inn_file.records.len());
        } else {
            warn!("Inn file not found: {}", path.display());
        }
        Ok(())
    }

    async fn load_item_file(&self, path: String) -> Result<(), Box<dyn std::error::Error + '_>> {
        let path = Path::new(&path);
        if Path::exists(path) {
            let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
            let mut item_file = self.item_file.lock()?;
            item_file.read(&mut raw_file)?;
            info!("{} items loaded", item_file.records.len());
        } else {
            warn!("Item file not found: {}", path.display());
        }
        Ok(())
    }

    async fn load_master_file(&self, path: String) -> Result<(), Box<dyn std::error::Error + '_>> {
        let path = Path::new(&path);
        if Path::exists(path) {
            let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
            let mut master_file = self.master_file.lock()?;
            master_file.read(&mut raw_file)?;
            info!("{} skill masters loaded", master_file.records.len());
        } else {
            warn!("Skill Master file not found: {}", path.display());
        }
        Ok(())
    }

    async fn load_npc_file(&self, path: String) -> Result<(), Box<dyn std::error::Error + '_>> {
        let path = Path::new(&path);
        if Path::exists(path) {
            let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
            let mut npc_file = self.npc_file.lock()?;
            npc_file.read(&mut raw_file)?;
            info!("{} npcs loaded", npc_file.records.len());
        } else {
            warn!("NPC file not found: {}", path.display());
        }
        Ok(())
    }

    async fn load_shop_file(&self, path: String) -> Result<(), Box<dyn std::error::Error + '_>> {
        let path = Path::new(&path);
        if Path::exists(path) {
            let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
            let mut shop_file = self.shop_file.lock()?;
            shop_file.read(&mut raw_file)?;
            info!("{} shops loaded", shop_file.records.len());
        } else {
            warn!("Shop file not found: {}", path.display());
        }
        Ok(())
    }

    async fn load_spell_file(&self, path: String) -> Result<(), Box<dyn std::error::Error + '_>> {
        let path = Path::new(&path);
        if Path::exists(path) {
            let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
            let mut spell_file = self.spell_file.lock()?;
            spell_file.read(&mut raw_file)?;
            info!("{} spells loaded", spell_file.records.len());
        } else {
            warn!("Spell file not found: {}", path.display());
        }
        Ok(())
    }

    async fn load_talk_file(&self, path: String) -> Result<(), Box<dyn std::error::Error + '_>> {
        let path = Path::new(&path);
        if Path::exists(path) {
            let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
            let mut talk_file = self.talk_file.lock()?;
            talk_file.read(&mut raw_file)?;
            info!("{} speech records loaded", talk_file.records.len());
        } else {
            warn!("Talk file not found: {}", path.display());
        }
        Ok(())
    }
}
