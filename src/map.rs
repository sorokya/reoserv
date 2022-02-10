use eo::data::map::MapFile;

use crate::character::Character;

#[derive(Debug)]
pub struct Map {
    pub characters: Vec<Character>,
    pub file: MapFile,
}

impl Map {
    pub fn new(file: MapFile) -> Self {
        Self {
            characters: Vec::new(),
            file,
        }
    }
}
