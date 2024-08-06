use eolib::protocol::{net::Item, Coords};

#[derive(Debug, Serialize)]
pub struct MapState {
    pub name: String,
    pub chests: Vec<MapStateChest>,
    pub npcs: Vec<MapStateNpc>,
    pub characters: Vec<MapStateCharacter>,
    pub items: Vec<MapStateItem>,
}

#[derive(Debug, Serialize)]
pub struct MapStateChest {
    pub coords: Coords,
    pub items: Vec<Item>,
}

#[derive(Debug, Serialize)]
pub struct MapStateItem {
    pub coords: Coords,
    pub index: i32,
    pub id: i32,
    pub amount: i32,
}

#[derive(Debug, Serialize)]
pub struct MapStateNpc {
    pub index: i32,
    pub id: i32,
    pub coords: Coords,
    pub alive: bool,
}

#[derive(Debug, Serialize)]
pub struct MapStateCharacter {
    pub id: i32,
    pub name: String,
    pub coords: Coords,
    pub level: i32,
    pub class: i32,
    pub guild: String,
}
