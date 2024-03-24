SELECT `quest_id`, `state`, `npc_kills`, `player_kills`
FROM `QuestProgress`
WHERE `character_id` = :character_id;
