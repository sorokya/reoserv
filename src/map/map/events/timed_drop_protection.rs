use super::super::Map;

impl Map {
    pub fn timed_drop_protection(&mut self) {
        for item in self.items.values_mut() {
            if item.protected_ticks > 0 {
                item.protected_ticks -= 1;
            }
        }
    }
}
