#[derive(Debug, Serialize)]
pub struct MapListItem {
    pub id: i32,
    pub name: String,
    pub players: i32,
    pub npcs: i32,
    pub items: i32,
}
