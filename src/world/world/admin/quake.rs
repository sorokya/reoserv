use super::super::World;

impl World {
    pub fn quake(&mut self, magnitude: i32) {
        let maps = match self.maps {
            Some(ref maps) => maps,
            None => return,
        };

        for map in maps.values() {
            map.quake(magnitude);
        }
    }
}
