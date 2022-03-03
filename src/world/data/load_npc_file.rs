use std::path::Path;

use eo::data::pubs::NPCFile;

pub async fn load_npc_file(
    path: String,
) -> Result<NPCFile, Box<dyn std::error::Error + Send + Sync>> {
    let path = Path::new(&path);
    if Path::exists(path) {
        let mut raw_file = tokio::fs::File::open(path).await?.into_std().await;
        let mut npc_file = NPCFile::new();
        npc_file.read(&mut raw_file)?;
        info!("{} npcs loaded", npc_file.records.len());
        Ok(npc_file)
    } else {
        warn!("NPC file not found: {}", path.display());
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "NPC file not found",
        )))
    }
}
