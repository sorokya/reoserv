use super::super::Map;

impl Map {
    pub fn recover_npcs(&mut self) {
        for npc in self.npcs.values_mut() {
            if npc.alive && npc.hp < npc.max_hp {
                npc.hp += (npc.max_hp / 10) + 1;
                if npc.hp > npc.max_hp {
                    npc.hp = npc.max_hp;
                }
            }
        }
    }
}
