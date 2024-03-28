SELECT `quest_id`, `state`, `npc_kills`, `player_kills`, `done_at`, `completions`
FROM `QuestProgress`
WHERE `character_id` = :character_id;
