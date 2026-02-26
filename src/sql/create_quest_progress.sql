INSERT INTO `character_quest_progress` (`character_id`, `quest_id`, `state`, `npc_kills`, `player_kills`, `done_at`, `completions`)
VALUES (:character_id, :quest_id, :state, :npc_kills, :player_kills, :done_at, :completions);
