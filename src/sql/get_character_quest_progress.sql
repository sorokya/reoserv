SELECT `quest_id`, `state`, `npc_kills`, `player_kills`, `done_at`, `completions`
FROM `character_quest_progress`
WHERE `character_id` = :character_id;
