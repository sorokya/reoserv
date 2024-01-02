use super::super::Map;

impl Map {
    pub fn jukebox_timer(&mut self) {
        if !self.has_jukebox {
            return;
        }

        if self.jukebox_ticks == 0 {
            self.jukebox_player = None;
            return;
        }

        self.jukebox_ticks -= 1;
    }
}
