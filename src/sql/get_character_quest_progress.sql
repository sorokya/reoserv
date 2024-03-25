SELECT `quest_id`, `state`, `npc_kills`, `player_kills`, `done`
FROM `QuestProgress`
WHERE `character_id` = :character_id;
